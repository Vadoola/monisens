use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::sync::RwLock;

use tokio::fs;
use tokio::io;
use tokio::io::AsyncRead;

use crate::controller::{self as ctrl, DeviceID};
use crate::{app, debug_from_display};

use super::db_model;

#[cfg(target_os = "macos")]
const MODULE_FILE_EXT: &str = ".dylib";

#[cfg(target_os = "linux")]
const MODULE_FILE_EXT: &str = ".so";

#[cfg(target_os = "windows")]
const MODULE_FILE_EXT: &str = ".dll";

#[derive(thiserror::Error)]
pub enum DeviceError {
    // TODO
    #[error("unknown device was provided in device_sensors")]
    DeviceSensorsUnknownDevice,
    #[error("unknown sensor data type in table '{0}' in column '{1}'")]
    SensorDataUnknownType(String, String),
    #[error("device with id '{0}' was not found. Most probably it was deleted")]
    DeviceNotFound(DeviceID),
    #[error("device with id '{0}' was not configured")]
    DeviceNotConfigured(DeviceID),
}

debug_from_display!(DeviceError);

pub struct Device {
    /// == `device.name` in DB
    name: String,
    display_name: String,
    module_dir: PathBuf,
    data_dir: PathBuf,
    init_state: ctrl::DeviceInitState,

    /// [`HashMap`]<`sensor's table name`, [`Sensor`]>
    sensor_map: HashMap<String, ctrl::Sensor>,
}

impl Device {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_display_name(&self) -> &String {
        &self.display_name
    }

    pub fn get_sensors(&self) -> &HashMap<String, ctrl::Sensor> {
        &self.sensor_map
    }
}

/// `DeviceManager` hosts data of all devices like names and data folders, sensors info etc.
#[derive(Clone)]
pub struct DeviceManager {
    last_id: Arc<AtomicI32>,
    device_map: Arc<RwLock<HashMap<DeviceID, Arc<RwLock<Device>>>>>,
    data_dir: Arc<PathBuf>,
}

impl DeviceManager {
    /// `new` method creates an internal map of devices based on the provided `devices` vector and associates
    /// the device with its sensors based on the information in `device_sensors` and `sensor_types`.
    pub fn new(
        devices: &Vec<db_model::Device>,
        device_sensors: &Vec<db_model::DeviceSensor>,
        sensor_types: &Vec<db_model::ColumnType>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut last_id: i32 = 0;
        let mut device_map: HashMap<DeviceID, Arc<RwLock<Device>>> =
            HashMap::with_capacity(devices.len());

        // Init devices
        for device in devices {
            device_map.insert(
                DeviceID::new(device.id),
                Arc::new(RwLock::new(Device {
                    name: device.name.clone(),
                    display_name: device.display_name.clone(),
                    module_dir: PathBuf::from_str(&device.module_dir)?,
                    data_dir: PathBuf::from_str(&device.data_dir)?,
                    sensor_map: HashMap::new(),
                    init_state: ctrl::DeviceInitState::from(&device.init_state),
                })),
            );

            if device.id > last_id {
                last_id = device.id;
            }
        }

        // Init all sensors
        let mut sensors_res: HashMap<String, ctrl::Sensor> =
            HashMap::with_capacity(device_sensors.len());

        for sensor_type in sensor_types {
            let sensor =
                sensors_res
                    .entry(sensor_type.table_name.clone())
                    .or_insert(ctrl::Sensor {
                        name: sensor_type.table_name.clone(),
                        data_map: HashMap::new(),
                    });

            let typ = sensor_data_type_from_udt(&sensor_type.udt_name).ok_or(
                DeviceError::SensorDataUnknownType(
                    sensor_type.table_name.clone(),
                    sensor_type.column_name.clone(),
                ),
            )?;

            sensor.data_map.insert(
                sensor_type.column_name.clone(),
                ctrl::SensorDataEntry {
                    name: sensor_type.column_name.clone(),
                    typ: typ,
                },
            );
        }

        // Map sensors to its devices
        for device_sensor in device_sensors {
            let device_id = DeviceID::new(device_sensor.device_id);

            let device = device_map
                .get(&device_id)
                .ok_or(DeviceError::DeviceSensorsUnknownDevice)?;

            if let Some(sensor) = sensors_res.remove(&device_sensor.sensor_table_name) {
                let mut device = device.write().unwrap();

                device
                    .sensor_map
                    .insert(device_sensor.sensor_name.clone(), sensor);
            }
        }

        let res = Self {
            last_id: Arc::new(AtomicI32::new(last_id)),
            device_map: Arc::new(RwLock::new(device_map)),
            data_dir: Arc::new(check_and_return_base_dir()),
        };

        // TODO: Replace with logger
        println!("Inited DeviceManager with data_dir = {:?}", res.data_dir);
        std::io::stdout().flush().unwrap();

        Ok(res)
    }

    /// `start_device_init` creates directories for device's data and module and writes
    /// module file to `<app_dir>/device/<id>-<device_name_snake_case>/module/` directory
    ///
    /// Created structure:
    /// ```
    /// <app_dir>/
    ///     device/
    ///         <id>-<device_name_snake_case>/
    ///             module/
    ///             data/
    /// ```
    pub async fn start_device_init<'f, F>(
        &self,
        name: String,
        display_name: String,
        module_file: &'f mut F,
    ) -> Result<ctrl::DeviceInitData, Box<dyn Error>>
    where
        F: AsyncRead + Unpin + ?Sized,
    {
        let id = self.inc_last_id();

        let dir_name = build_device_dir_name(&id, &name);
        self.create_data_dir(&dir_name).await?;

        let module_dir = dir_name.join("module");
        self.create_data_dir(&module_dir).await?;

        let data_dir = dir_name.join("data");
        self.create_data_dir(&data_dir).await?;

        let full_module_path = self.full_module_file_path(&module_dir);
        create_file(&full_module_path, module_file).await?;

        let device = Device {
            name,
            display_name,
            module_dir: module_dir.clone(),
            data_dir: data_dir.clone(),
            sensor_map: Default::default(),
            init_state: ctrl::DeviceInitState::Device,
        };

        (*self.device_map.write().unwrap()).insert(id, Arc::new(RwLock::new(device)));

        Ok(ctrl::DeviceInitData {
            id,
            module_file: full_module_path,
            data_dir: data_dir.clone(),
            full_data_dir: self.full_data_dir(&data_dir),
            module_dir,
            init_state: ctrl::DeviceInitState::Device,
        })
    }

    pub fn device_sensor_init(
        &self,
        device_id: &DeviceID,
        sensors: Vec<ctrl::Sensor>,
    ) -> Result<(), DeviceError> {
        let device = self.get_device(device_id)?;
        let mut device = device.write().unwrap();
        for sensor in sensors {
            device.sensor_map.insert(sensor.name.clone(), sensor);
        }
        device.init_state = ctrl::DeviceInitState::Sensors;

        Ok(())
    }

    pub fn get_device_name(&self, id: &DeviceID) -> Result<String, DeviceError> {
        let device = self.get_device(id)?;
        let device = device.read().unwrap();

        Ok(device.name.clone())
    }

    pub fn get_device_init_state(
        &self,
        id: DeviceID,
    ) -> Result<ctrl::DeviceInitState, DeviceError> {
        let device = self.get_device(&id)?;
        let device = device.read().unwrap();

        Ok(device.init_state.clone())
    }

    pub async fn delete_device(&self, id: &DeviceID) -> Result<(), Box<dyn Error>> {
        let mut device_map = self.device_map.write().unwrap();

        let device = device_map
            .get(id)
            .ok_or(DeviceError::DeviceNotFound(id.clone()))?
            .clone();

        // Intentionally lock device for write 'cause we're deleting it
        let device = device.write().unwrap();

        let device_dir = self.data_dir.join(build_device_dir_name(id, &device.name));
        fs::remove_dir_all(device_dir).await?;

        device_map.remove(id);

        Ok(())
    }

    pub fn get_device_ids(&self) -> Vec<DeviceID> {
        self.device_map.read().unwrap().keys().copied().collect()
    }

    pub fn get_init_data_all_devices(&self) -> Vec<ctrl::DeviceInitData> {
        let device_map = self.device_map.read().unwrap();
        let mut res = Vec::with_capacity(device_map.len());
        for (id, data_handler) in device_map.iter() {
            let data = data_handler.read().unwrap();

            res.push(ctrl::DeviceInitData {
                id: id.clone(),
                module_dir: data.module_dir.clone(),
                data_dir: data.data_dir.clone(),
                full_data_dir: self.full_data_dir(&data.data_dir),
                module_file: self.full_module_file_path(&data.module_dir),
                init_state: data.init_state.clone(),
            })
        }

        res
    }

    /// get_device_info_list returns an unsorted list of devices.
    ///
    /// Devices must be configured to be returned (`init_state == DeviceInitState::Sensors`)
    pub fn get_device_info_list(&self) -> Vec<ctrl::DeviceInfo> {
        let device_map = self.device_map.read().unwrap();
        let mut res = Vec::with_capacity(device_map.len());
        for (id, data_handler) in device_map.iter() {
            let data = data_handler.read().unwrap();

            if data.init_state == ctrl::DeviceInitState::Sensors {
                res.push(ctrl::DeviceInfo {
                    id: id.clone(),
                    display_name: data.get_display_name().clone(),
                })
            }
        }

        res
    }

    /// get_device_info_list returns list of device's sensors and their data types.
    ///
    /// Both sensors and their data types are not sorted.
    ///
    /// If the device is not configured, an error `DeviceError::DeviceNotConfigured` is returned.
    pub fn get_device_sensor_info(
        &self,
        device_id: DeviceID,
    ) -> Result<Vec<ctrl::SensorInfo>, DeviceError> {
        let device = self.get_device(&device_id)?;
        let device = device.read().unwrap();

        if device.init_state != ctrl::DeviceInitState::Sensors {
            return Err(DeviceError::DeviceNotConfigured(device_id));
        }

        Ok(device
            .sensor_map
            .iter()
            .map(|(name, sensor)| ctrl::SensorInfo {
                name: name.clone(),
                data: sensor
                    .data_map
                    .iter()
                    .map(|(name, data)| ctrl::SensorDataEntry {
                        name: name.clone(),
                        typ: data.typ.clone(),
                    })
                    .collect(),
            })
            .collect())
    }

    fn inc_last_id(&self) -> DeviceID {
        let prev_last_id = self.last_id.fetch_add(1, Ordering::SeqCst);

        DeviceID::new(prev_last_id + 1)
    }

    fn get_device(&self, id: &DeviceID) -> Result<Arc<RwLock<Device>>, DeviceError> {
        if let Some(device) = self.device_map.read().unwrap().get(id) {
            Ok(device.clone())
        } else {
            Err(DeviceError::DeviceNotFound(id.clone()))
        }
    }

    async fn create_data_dir<P: AsRef<Path>>(&self, rel_path: P) -> io::Result<()> {
        fs::create_dir(self.full_data_dir(rel_path)).await
    }

    fn full_data_dir<P: AsRef<Path>>(&self, data_dir: P) -> PathBuf {
        (*self.data_dir).join(data_dir)
    }

    fn full_module_file_path<P: AsRef<Path>>(&self, module_dir: P) -> PathBuf {
        let mut p = (*self.data_dir).join(module_dir);
        p.push("lib".to_string() + MODULE_FILE_EXT);

        p
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self {
            last_id: Default::default(),
            device_map: Default::default(),
            data_dir: Arc::new(check_and_return_base_dir()),
        }
    }
}

fn check_and_return_base_dir() -> PathBuf {
    println!("Initializing base dir...");
    std::io::stdout().flush().unwrap();

    let path = app::data_dir().join("device");

    let p = Path::new(&path);

    if !p.is_dir() {
        std::fs::create_dir(p).expect(&format!("failed to create base dir: '{path:?}'",));
    }

    path
}

fn build_device_dir_name(id: &DeviceID, name: &String) -> PathBuf {
    PathBuf::from_str(&(id.get_raw().to_string() + "-" + &name)).unwrap()
}

async fn create_file<'a, R: AsyncRead + Unpin + ?Sized, P: AsRef<Path>>(
    path: P,
    data: &'a mut R,
) -> io::Result<()> {
    if let Ok(_) = fs::File::open(&path).await {
        return Err(io::ErrorKind::AlreadyExists.into());
    }

    let mut file = fs::File::create(path).await?;
    io::copy(data, &mut file).await?;

    Ok(())
}

fn sensor_data_type_from_udt(udt_name: &str) -> Option<ctrl::SensorDataType> {
    match udt_name {
        "int2" => Some(ctrl::SensorDataType::Int16),
        "int4" => Some(ctrl::SensorDataType::Int32),
        "int8" => Some(ctrl::SensorDataType::Int64),
        "float4" => Some(ctrl::SensorDataType::Float32),
        "float8" => Some(ctrl::SensorDataType::Float64),
        "timestamp" => Some(ctrl::SensorDataType::Timestamp),
        "text" => Some(ctrl::SensorDataType::String),
        "jsonb" => Some(ctrl::SensorDataType::JSON),
        _ => None,
    }
}
