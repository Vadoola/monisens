use std::error::Error;
use tokio::io::AsyncRead;

use super::super::model;

pub trait IService: Sync + Send + Clone {
    /// `start_device_init` starts device initialization. It initializes directory
    /// for device's data, saves device's module there and saves device info in a storage.
    ///
    /// It must set device's init state to `Device`
    async fn start_device_init<'f, F: AsyncRead + Unpin + ?Sized>(
        &self,
        display_name: String,
        module_file: &'f mut F,
    ) -> Result<model::DeviceInitData, Box<dyn Error>>;

    /// `device_sensor_init` initializes device's sensors by saving them in a storage.
    ///
    /// It must set device's init state to `Sensors`
    async fn device_sensor_init(
        &self,
        device_id: model::DeviceID,
        sensors: Vec<model::Sensor>,
    ) -> Result<(), Box<dyn Error>>;

    /// `interrupt_device_init` interrupts device initialization.
    ///
    /// It must ensure that device's init state is `Device`.
    ///
    /// It must delete device's data from disk and storage.
    async fn interrupt_device_init(&self, id: model::DeviceID) -> Result<(), Box<dyn Error>>;

    /// `get_device_ids` returns all device ids.
    fn get_device_ids(&self) -> Vec<model::DeviceID>;

    /// `get_init_data_all_devices` returns all devices' data.
    fn get_init_data_all_devices(&self) -> Vec<model::DeviceInitData>;

    /// `save_sensor_data` saves sensor data for device.
    async fn save_sensor_data(
        &self,
        id: model::DeviceID,
        msg: model::SensorMsg,
    ) -> Result<(), Box<dyn Error>>;

    /// `get_sensor_data` returns sensor data for device.
    async fn get_sensor_data(
        &self,
        id: model::DeviceID,
        sensor_name: String,
        fields: Vec<String>,
        filter: model::SensorDataFilter,
    ) -> Result<Vec<model::SensorDataList>, Box<dyn Error>>;

    /// `get_device_info_list` returns device info list.
    fn get_device_info_list(&self) -> Vec<model::DeviceInfo>;

    /// `get_device_sensor_info` returns device sensor info.
    fn get_device_sensor_info(
        &self,
        device_id: model::DeviceID,
    ) -> Result<Vec<model::SensorInfo>, Box<dyn Error>>;

    /// `save_monitor_conf` saves monitoring config.
    async fn save_monitor_conf(
        &self,
        monitor_conf: model::MonitorConf,
    ) -> Result<i32, Box<dyn Error>>;

    /// `get_monitor_conf_list` returns a list of monitoring configs.
    async fn get_monitor_conf_list(
        &self,
        filter: model::MonitorConfListFilter,
    ) -> Result<Vec<model::MonitorConf>, Box<dyn Error>>;
}
