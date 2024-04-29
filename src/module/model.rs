use super::bindings_gen as bg;
use super::error::{ComError, ModuleError};
use super::conv;

use libc::c_void;
use std::collections::HashMap;

use crate::controller;
use crate::controller::interface::module::MsgHandler;

pub const VERSION: u8 = 1;

pub type DeficeInfoRec = Result<Vec<controller::ConnParamConf>, ModuleError>;

fn device_connect_info(res: *mut DeficeInfoRec, info: *const bg::DeviceConnectInfo) {
    if info.is_null() {
        unsafe {
            *res = Err(ModuleError::InvalidPointer("device_connect_info"));
        }
        return;
    }

    if unsafe { (*info).connection_params }.is_null() {
        unsafe {
            *res = Err(ModuleError::InvalidPointer(
                "device_connect_info.connection_params",
            ));
        }
        return;
    }

    let len = unsafe { (*info).connection_params_len as usize };
    let mut device_connect_info: Vec<controller::ConnParamConf> = Vec::with_capacity(len);

    let params = unsafe { std::slice::from_raw_parts((*info).connection_params, len) };

    for param in params {
        if param.name.is_null() {
            unsafe {
                *res = Err(ModuleError::InvalidPointer(
                    "device_connect_info.connection_params[i].name",
                ));
            }
            return;
        }

        let name = conv::str_from_c_char(param.name);

        let info = match param.typ {
            bg::ConnParamType::ConnParamChoiceList => {
                let raw_info = param.info as *const bg::ConnParamChoiceListInfo;

                let choices = unsafe {
                    std::slice::from_raw_parts(
                        (*raw_info).choices,
                        (*raw_info).chioces_len as usize,
                    )
                };

                let res = controller::ConnParamChoiceListInfo {
                    choices: choices.iter().map(|v| conv::str_from_c_char(*v)).collect(),
                };

                Some(controller::ConnParamEntryInfo::ChoiceList(res))
            }
            _ => None,
        };

        device_connect_info.push(controller::ConnParamConf {
            name: name,
            typ: conv::bg_conn_param_type_to_ctrl(&param.typ),
            info,
        });
    }

    unsafe {
        *res = Ok(device_connect_info);
    }
}

pub struct Handle(*const c_void);

impl Handle {
    pub fn new() -> Self {
        Handle(std::ptr::null())
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    pub fn handler_ptr(&mut self) -> *mut *mut c_void {
        self as *mut Self as *mut *mut c_void
    }

    pub fn handler(&mut self) -> *mut c_void {
        self.0 as _
    }
}

unsafe impl Send for Handle {}

pub extern "C" fn device_info_callback(obj: *mut c_void, info: *mut bg::DeviceConnectInfo) {
    device_connect_info(obj as _, info);
}

fn build_device_conf_info(
    info: *mut bg::DeviceConfInfo,
) -> Result<controller::DeviceConfInfo, ModuleError> {
    if unsafe { (*info).device_confs }.is_null() {
        return Err(ModuleError::InvalidPointer("device_conf_info.device_confs"));
    }

    let confs =
        unsafe { std::slice::from_raw_parts((*info).device_confs, (*info).device_confs_len as _) };
    let mut res = controller::DeviceConfInfo::with_capacity(unsafe { (*info).device_confs_len } as _);

    for conf in confs {
        let data = build_device_conf_info_entry_data(conf)?;

        res.push(controller::DeviceConfInfoEntry {
            id: conf.id,
            name: conv::str_from_c_char(conf.name),
            data: data,
        });
    }

    Ok(res)
}

fn build_device_conf_info_entry_data(
    conf: &bg::DeviceConfInfoEntry,
) -> Result<controller::DeviceConfInfoEntryType, ModuleError> {
    match conf.typ {
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeSection => {
            let section = build_device_conf_info(conf.data as *mut bg::DeviceConfInfo)?;

            Ok(controller::DeviceConfInfoEntryType::Section(section))
        }
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeString => {
            let data = unsafe { *(conf.data as *mut bg::DeviceConfInfoEntryString) };

            Ok(controller::DeviceConfInfoEntryType::String(controller::DeviceConfInfoEntryString {
                required: data.required,
                default: conv::option_str_from_c_char(data.def),
                min_len: nullable_into_option(data.min_len),
                max_len: nullable_into_option(data.max_len),
                match_regex: conv::option_str_from_c_char(data.match_regex),
            }))
        }
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeInt => {
            let data = unsafe { *(conf.data as *mut bg::DeviceConfInfoEntryInt) };

            Ok(controller::DeviceConfInfoEntryType::Int(controller::DeviceConfInfoEntryInt {
                required: data.required,
                default: nullable_into_option(data.def),
                lt: nullable_into_option(data.lt),
                gt: nullable_into_option(data.gt),
                neq: nullable_into_option(data.neq),
            }))
        }
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeIntRange => {
            let data = unsafe { *(conf.data as *mut bg::DeviceConfInfoEntryIntRange) };

            Ok(controller::DeviceConfInfoEntryType::IntRange(
                controller::DeviceConfInfoEntryIntRange {
                    required: data.required,
                    def_from: nullable_into_option(data.def_from),
                    def_to: nullable_into_option(data.def_to),
                    min: data.min,
                    max: data.max,
                },
            ))
        }
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeFloat => {
            let data = unsafe { *(conf.data as *mut bg::DeviceConfInfoEntryFloat) };

            Ok(controller::DeviceConfInfoEntryType::Float(controller::DeviceConfInfoEntryFloat {
                required: data.required,
                default: nullable_into_option(data.def),
                lt: nullable_into_option(data.lt),
                gt: nullable_into_option(data.gt),
                neq: nullable_into_option(data.neq),
            }))
        }
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeFloatRange => {
            let data = unsafe { *(conf.data as *mut bg::DeviceConfInfoEntryFloatRange) };

            Ok(controller::DeviceConfInfoEntryType::FloatRange(
                controller::DeviceConfInfoEntryFloatRange {
                    required: data.required,
                    def_from: nullable_into_option(data.def_from),
                    def_to: nullable_into_option(data.def_to),
                    min: data.min,
                    max: data.max,
                },
            ))
        }
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeJSON => {
            let data = unsafe { *(conf.data as *mut bg::DeviceConfInfoEntryJSON) };

            Ok(controller::DeviceConfInfoEntryType::JSON(controller::DeviceConfInfoEntryJSON {
                required: data.required,
                default: conv::option_str_from_c_char(data.def),
            }))
        }
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeChoiceList => {
            let data = unsafe { *(conf.data as *mut bg::DeviceConfInfoEntryChoiceList) };

            let mut entry = controller::DeviceConfInfoEntryChoiceList {
                required: data.required,
                default: nullable_into_option(data.def),
                choices: Vec::with_capacity(data.chioces_len as _),
            };

            for choice in unsafe { std::slice::from_raw_parts(data.choices, data.chioces_len as _) }
            {
                entry.choices.push(conv::str_from_c_char(*choice));
            }

            Ok(controller::DeviceConfInfoEntryType::ChoiceList(entry))
        }
    }
}

pub type DeviceConfInfoRec = Result<controller::DeviceConfInfo, ModuleError>;

fn device_conf_info(res: *mut DeviceConfInfoRec, info: *mut bg::DeviceConfInfo) {
    if info.is_null() {
        unsafe {
            *res = Err(ModuleError::InvalidPointer("device_conf"));
        }
        return;
    }

    unsafe {
        *res = build_device_conf_info(info);
    }
}

pub extern "C" fn device_conf_info_callback(obj: *mut c_void, info: *mut bg::DeviceConfInfo) {
    device_conf_info(obj as _, info);
}

pub fn build_device_conf(confs: &Vec<bg::DeviceConfEntry>) -> bg::DeviceConf {
    bg::DeviceConf {
        confs: confs.as_ptr() as _,
        confs_len: confs.len() as _,
    }
}

pub fn bg_sensor_type_infos_to_sensor_vec(
    infos: *mut bg::SensorTypeInfos,
) -> Result<Vec<controller::Sensor>, ModuleError> {
    let infos_slice = unsafe {
        std::slice::from_raw_parts(
            (*infos).sensor_type_infos,
            (*infos).sensor_type_infos_len as _,
        )
    };

    let mut res_infos = Vec::with_capacity(infos_slice.len());
    for info in infos_slice {
        let data_type_infos_slice = unsafe {
            std::slice::from_raw_parts(info.data_type_infos, info.data_type_infos_len as _)
        };

        let mut res_data_type_infos_map = HashMap::with_capacity(data_type_infos_slice.len());
        for data_type_info in data_type_infos_slice {
            let name = conv::str_from_c_char(data_type_info.name);
            res_data_type_infos_map.insert(name.clone(), controller::SensorDataEntry {
                name,
                typ: conv::bg_sensor_data_type_to_ctrl(&data_type_info.typ),
            });
        }

        res_infos.push(controller::Sensor {
            name: conv::str_from_c_char(info.name),
            data_map: res_data_type_infos_map,
        })
    }

    Ok(res_infos)
}

pub type SensorTypeInfosRec = Result<Vec<controller::Sensor>, ModuleError>;

fn sensor_type_infos(res: *mut SensorTypeInfosRec, infos: *mut bg::SensorTypeInfos) {
    if infos.is_null() {
        unsafe {
            *res = Err(ModuleError::InvalidPointer("device_conf"));
        }
        return;
    }

    unsafe {
        *res = bg_sensor_type_infos_to_sensor_vec(infos);
    }
}

pub extern "C" fn sensor_type_infos_callback(obj: *mut c_void, infos: *mut bg::SensorTypeInfos) {
    sensor_type_infos(obj as _, infos);
}

pub struct MsgHandle(Box<dyn MsgHandler>);

impl MsgHandle {
    pub fn new<H: MsgHandler + 'static>(msg_handler: H) -> Self {
        Self(Box::new(msg_handler))
    }
}

pub extern "C" fn handle_msg_callback(handler: *mut c_void, msg_data: bg::Message) {
    let h = handler as *const MsgHandle;

    unsafe {
        let data = conv::bg_message_to_ctrl(&msg_data);
        let h = &(*h).0;
        h.handle_msg(data);
    }
}

// ------------------- Utility functions -------------------

pub fn  convert_com_error(err: u8) -> Result<(), ComError> {
    match err {
        0 => Ok(()),
        1 => Err(ComError::ConnectionError),
        2 => Err(ComError::InvalidArgument),
        _ => Err(ComError::Unknown),
    }
}

fn nullable_into_option<T: Copy>(nullable_val: *mut T) -> Option<T> {
    if nullable_val.is_null() {
        None
    } else {
        Some(unsafe { *nullable_val })
    }
}
