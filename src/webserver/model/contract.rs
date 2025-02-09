use std::collections::HashMap;

use crate::controller;
use actix_multipart::form::{bytes::Bytes, tempfile::TempFile, text::Text, MultipartForm};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, MultipartForm, ToSchema)]
pub struct TestUploadForm {
    #[schema(value_type = String, format = Binary)]
    #[multipart(rename = "file")]
    pub file: Bytes,
    #[schema(value_type = String, format = Byte)]
    pub name: Text<String>,
}

#[derive(Debug, MultipartForm, ToSchema)]
pub struct DeviceStartInitRequest {
    #[schema(value_type = String, format = Byte)]
    pub device_name: Text<String>,
    #[schema(value_type = String, format = Binary)]
    pub module_file: TempFile,
}

#[derive(Serialize, ToSchema)]
pub struct DeviceStartInitResponse {
    pub device_id: i32,
    pub conn_params: Vec<ConfInfoEntry>,
}

impl From<controller::DeviceConnData> for DeviceStartInitResponse {
    fn from(mut value: controller::DeviceConnData) -> Self {
        Self {
            device_id: value.id.get_raw(),
            conn_params: value.conn_params.drain(..).map(|v| v.into()).collect(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConnectDeviceRequest {
    pub device_id: i32,
    pub connect_conf: Vec<ConfEntry>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ObtainDeviceConfInfoRequest {
    pub device_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ObtainDeviceConfInfoResponse {
    pub device_conf_info: Vec<ConfInfoEntry>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConfInfoEntry {
    pub id: i32,
    pub name: String,
    pub data: ConfInfoEntryType,
}

impl From<controller::ConfInfoEntry> for ConfInfoEntry {
    fn from(value: controller::ConfInfoEntry) -> Self {
        Self {
            id: value.id,
            name: value.name,
            data: value.data.into(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub enum ConfInfoEntryType {
    String(ConfInfoEntryString),
    Int(ConfInfoEntryInt),
    IntRange(ConfInfoEntryIntRange),
    Float(ConfInfoEntryFloat),
    FloatRange(ConfInfoEntryFloatRange),
    JSON(ConfInfoEntryJSON),
    ChoiceList(ConfInfoEntryChoiceList),
    Section(Vec<ConfInfoEntry>),
}

impl From<controller::ConfInfoEntryType> for ConfInfoEntryType {
    fn from(value: controller::ConfInfoEntryType) -> Self {
        match value {
            controller::ConfInfoEntryType::Section(mut v) => {
                ConfInfoEntryType::Section(v.drain(..).map(|vv| vv.into()).collect())
            }
            controller::ConfInfoEntryType::String(v) => {
                ConfInfoEntryType::String(ConfInfoEntryString {
                    required: v.required,
                    default: v.default,
                    min_len: v.min_len,
                    max_len: v.max_len,
                    match_regex: v.match_regex,
                })
            }
            controller::ConfInfoEntryType::Int(v) => ConfInfoEntryType::Int(ConfInfoEntryInt {
                required: v.required,
                default: v.default,
                lt: v.lt,
                gt: v.gt,
                neq: v.neq,
            }),
            controller::ConfInfoEntryType::IntRange(v) => {
                ConfInfoEntryType::IntRange(ConfInfoEntryIntRange {
                    required: v.required,
                    def_from: v.def_from,
                    def_to: v.def_to,
                    min: v.min,
                    max: v.max,
                })
            }
            controller::ConfInfoEntryType::Float(v) => {
                ConfInfoEntryType::Float(ConfInfoEntryFloat {
                    required: v.required,
                    default: v.default,
                    lt: v.lt,
                    gt: v.gt,
                    neq: v.neq,
                })
            }
            controller::ConfInfoEntryType::FloatRange(v) => {
                ConfInfoEntryType::FloatRange(ConfInfoEntryFloatRange {
                    required: v.required,
                    def_from: v.def_from,
                    def_to: v.def_to,
                    min: v.min,
                    max: v.max,
                })
            }
            controller::ConfInfoEntryType::JSON(v) => ConfInfoEntryType::JSON(ConfInfoEntryJSON {
                required: v.required,
                default: v.default,
            }),
            controller::ConfInfoEntryType::ChoiceList(v) => {
                ConfInfoEntryType::ChoiceList(ConfInfoEntryChoiceList {
                    required: v.required,
                    default: v.default,
                    choices: v.choices,
                })
            }
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConfInfoEntryString {
    pub required: bool,
    pub default: Option<String>,

    pub min_len: Option<i32>,
    pub max_len: Option<i32>,
    pub match_regex: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConfInfoEntryInt {
    pub required: bool,
    pub default: Option<i32>,

    pub lt: Option<i32>,
    pub gt: Option<i32>,
    pub neq: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConfInfoEntryIntRange {
    pub required: bool,
    pub def_from: Option<i32>,
    pub def_to: Option<i32>,

    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConfInfoEntryFloat {
    pub required: bool,
    pub default: Option<f32>,

    pub lt: Option<f32>,
    pub gt: Option<f32>,
    pub neq: Option<f32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConfInfoEntryFloatRange {
    pub required: bool,
    pub def_from: Option<f32>,
    pub def_to: Option<f32>,

    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConfInfoEntryJSON {
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConfInfoEntryChoiceList {
    pub required: bool,
    pub default: Option<i32>,

    pub choices: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConfigureDeviceRequest {
    pub device_id: i32,
    pub confs: Vec<ConfEntry>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConfEntry {
    pub id: i32,
    pub data: Option<ConfType>,
}

impl From<ConfEntry> for controller::ConfEntry {
    fn from(value: ConfEntry) -> Self {
        controller::ConfEntry {
            id: value.id,
            data: value.data.map(|v| v.into()),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub enum ConfType {
    String(String),
    Int(i32),
    IntRange([i32; 2]),
    Float(f32),
    FloatRange([f32; 2]),
    JSON(String),
    ChoiceList(i32),
}

impl From<ConfType> for controller::ConfType {
    fn from(value: ConfType) -> Self {
        match value {
            ConfType::String(v) => controller::ConfType::String(v),
            ConfType::Int(v) => controller::ConfType::Int(v),
            ConfType::IntRange(v) => controller::ConfType::IntRange(v),
            ConfType::Float(v) => controller::ConfType::Float(v),
            ConfType::FloatRange(v) => controller::ConfType::FloatRange(v),
            ConfType::JSON(v) => controller::ConfType::JSON(v),
            ConfType::ChoiceList(v) => controller::ConfType::ChoiceList(v),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct InterruptDeviceInitRequest {
    pub device_id: i32,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct GetSensorDataRequest {
    #[validate(range(min = 1))]
    pub device_id: i32,
    #[validate(length(min = 1))]
    pub sensor: String,
    #[validate(length(min = 1))]
    pub fields: Vec<String>,
    #[validate]
    pub sort: Sort,
    pub from: Option<SensorData>,
    #[validate(range(max = 1000))]
    pub limit: Option<i32>,
}

impl From<GetSensorDataRequest> for controller::GetSensorDataPayload {
    fn from(value: GetSensorDataRequest) -> Self {
        Self {
            device_id: value.device_id,
            sensor: value.sensor,
            fields: value.fields,
            sort: value.sort.into(),
            from: value.from.map(|v| v.into()),
            limit: value.limit,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetSensorDataResponse {
    result: Vec<HashMap<String, SensorData>>,
}

impl From<controller::GetSensorDataResult> for GetSensorDataResponse {
    fn from(mut value: controller::GetSensorDataResult) -> Self {
        Self {
            result: value
                .drain(..)
                .map(|mut v| {
                    v.drain()
                        .map(|(field, val)| (field, val.data.into()))
                        .collect()
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Validate, Deserialize, ToSchema)]
pub struct Sort {
    #[validate(length(min = 1))]
    pub field: String,
    pub order: SortDir,
}

impl From<Sort> for controller::Sort {
    fn from(value: Sort) -> Self {
        Self {
            field: value.field,
            order: value.order.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub enum SensorData {
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    #[schema(value_type = String)]
    Timestamp(chrono::NaiveDateTime),
    String(String),
    JSON(String),
}

impl From<SensorData> for controller::SensorDataTypeValue {
    fn from(value: SensorData) -> Self {
        match value {
            SensorData::Int16(v) => controller::SensorDataTypeValue::Int16(v),
            SensorData::Int32(v) => controller::SensorDataTypeValue::Int32(v),
            SensorData::Int64(v) => controller::SensorDataTypeValue::Int64(v),
            SensorData::Float32(v) => controller::SensorDataTypeValue::Float32(v),
            SensorData::Float64(v) => controller::SensorDataTypeValue::Float64(v),
            SensorData::Timestamp(v) => controller::SensorDataTypeValue::Timestamp(v),
            SensorData::String(v) => controller::SensorDataTypeValue::String(v),
            SensorData::JSON(v) => controller::SensorDataTypeValue::JSON(v),
        }
    }
}

impl From<controller::SensorDataTypeValue> for SensorData {
    fn from(value: controller::SensorDataTypeValue) -> Self {
        match value {
            controller::SensorDataTypeValue::Int16(v) => SensorData::Int16(v),
            controller::SensorDataTypeValue::Int32(v) => SensorData::Int32(v),
            controller::SensorDataTypeValue::Int64(v) => SensorData::Int64(v),
            controller::SensorDataTypeValue::Float32(v) => SensorData::Float32(v),
            controller::SensorDataTypeValue::Float64(v) => SensorData::Float64(v),
            controller::SensorDataTypeValue::Timestamp(v) => SensorData::Timestamp(v),
            controller::SensorDataTypeValue::String(v) => SensorData::String(v),
            controller::SensorDataTypeValue::JSON(v) => SensorData::JSON(v),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetDeviceListResponse {
    result: Vec<DeviceEntry>,
}

impl From<Vec<controller::DeviceInfo>> for GetDeviceListResponse {
    fn from(mut value: Vec<controller::DeviceInfo>) -> Self {
        Self {
            result: value.drain(..).map(|v| v.into()).collect(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceEntry {
    pub id: i32,
    pub name: String,
}

impl From<controller::DeviceInfo> for DeviceEntry {
    fn from(value: controller::DeviceInfo) -> Self {
        Self {
            id: value.id.get_raw(),
            name: value.display_name,
        }
    }
}

#[derive(Clone, Debug, Validate, Deserialize, ToSchema)]
pub struct GetDeviceSensorInfoRequest {
    #[validate(range(min = 1))]
    pub device_id: i32,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct GetDeviceSensorInfoResponse {
    pub device_sensor_info: Vec<SensorInfo>,
}

impl From<Vec<controller::SensorInfo>> for GetDeviceSensorInfoResponse {
    fn from(mut value: Vec<controller::SensorInfo>) -> Self {
        let mut device_sensor_info: Vec<SensorInfo> = value.drain(..).map(|v| v.into()).collect();

        device_sensor_info.sort_unstable_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

        Self { device_sensor_info }
    }
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct SensorInfo {
    pub name: String,
    pub data: Vec<SensorDataInfo>,
}

impl From<controller::SensorInfo> for SensorInfo {
    fn from(mut value: controller::SensorInfo) -> Self {
        let mut data: Vec<SensorDataInfo> = value.data.drain(..).map(|v| v.into()).collect();

        data.sort_unstable_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

        Self {
            name: value.name,
            data: data,
        }
    }
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct SensorDataInfo {
    pub name: String,
    pub typ: SensorDataType,
}

impl From<controller::SensorDataEntry> for SensorDataInfo {
    fn from(value: controller::SensorDataEntry) -> Self {
        Self {
            name: value.name,
            typ: value.typ.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub enum SensorDataType {
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Timestamp,
    String,
    JSON,
}

impl From<controller::SensorDataType> for SensorDataType {
    fn from(value: controller::SensorDataType) -> Self {
        match value {
            controller::SensorDataType::Int16 => SensorDataType::Int16,
            controller::SensorDataType::Int32 => SensorDataType::Int32,
            controller::SensorDataType::Int64 => SensorDataType::Int64,
            controller::SensorDataType::Float32 => SensorDataType::Float32,
            controller::SensorDataType::Float64 => SensorDataType::Float64,
            controller::SensorDataType::Timestamp => SensorDataType::Timestamp,
            controller::SensorDataType::String => SensorDataType::String,
            controller::SensorDataType::JSON => SensorDataType::JSON,
        }
    }
}

#[derive(Clone, Debug, Validate, Deserialize, ToSchema)]
pub struct SaveMonitorConfRequest {
    #[validate(range(min = 1))]
    pub device_id: i32,
    #[validate(length(min = 1))]
    pub sensor: String,
    pub typ: MonitorType,
    pub config: MonitorTypeConf,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct SaveMonitorConfResponse {
    pub id: i32,
}

impl From<SaveMonitorConfRequest> for controller::MonitorConf {
    fn from(value: SaveMonitorConfRequest) -> Self {
        Self {
            id: 0,
            device_id: value.device_id,
            sensor: value.sensor,
            typ: value.typ.into(),
            config: value.config.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub enum MonitorType {
    Log,
    Line,
}

impl From<MonitorType> for controller::MonitorType {
    fn from(value: MonitorType) -> Self {
        match value {
            MonitorType::Log => controller::MonitorType::Log,
            MonitorType::Line => controller::MonitorType::Line,
        }
    }
}

impl From<controller::MonitorType> for MonitorType {
    fn from(value: controller::MonitorType) -> Self {
        match value {
            controller::MonitorType::Log => MonitorType::Log,
            controller::MonitorType::Line => MonitorType::Line,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub enum MonitorTypeConf {
    Log(MonitorLogConf),
    Line(MonitorLineConf),
}

impl From<MonitorTypeConf> for controller::MonitorTypeConf {
    fn from(value: MonitorTypeConf) -> Self {
        match value {
            MonitorTypeConf::Log(v) => controller::MonitorTypeConf::Log(v.into()),
            MonitorTypeConf::Line(v) => controller::MonitorTypeConf::Line(v.into()),
        }
    }
}

impl From<controller::MonitorTypeConf> for MonitorTypeConf {
    fn from(value: controller::MonitorTypeConf) -> Self {
        match value {
            controller::MonitorTypeConf::Log(v) => MonitorTypeConf::Log(v.into()),
            controller::MonitorTypeConf::Line(v) => MonitorTypeConf::Line(v.into()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct MonitorLogConf {
    pub fields: Vec<String>,
    pub sort_field: String,
    pub sort_direction: SortDir,
    pub limit: i32,
}

impl From<MonitorLogConf> for controller::MonitorLogConf {
    fn from(value: MonitorLogConf) -> Self {
        Self {
            fields: value.fields,
            sort_field: value.sort_field,
            sort_direction: value.sort_direction.into(),
            limit: value.limit,
        }
    }
}

impl From<controller::MonitorLogConf> for MonitorLogConf {
    fn from(value: controller::MonitorLogConf) -> Self {
        Self {
            fields: value.fields,
            sort_field: value.sort_field,
            sort_direction: value.sort_direction.into(),
            limit: value.limit,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct MonitorLineConf {
    pub x_field: String,
    pub y_field: String,
    pub limit: i32,
}

impl From<MonitorLineConf> for controller::MonitorLineConf {
    fn from(value: MonitorLineConf) -> Self {
        Self {
            x_field: value.x_field,
            y_field: value.y_field,
            limit: value.limit,
        }
    }
}

impl From<controller::MonitorLineConf> for MonitorLineConf {
    fn from(value: controller::MonitorLineConf) -> Self {
        Self {
            x_field: value.x_field,
            y_field: value.y_field,
            limit: value.limit,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub enum SortDir {
    ASC,
    DESC,
}

impl From<SortDir> for controller::SortDir {
    fn from(value: SortDir) -> Self {
        match value {
            SortDir::ASC => controller::SortDir::ASC,
            SortDir::DESC => controller::SortDir::DESC,
        }
    }
}

impl From<controller::SortDir> for SortDir {
    fn from(value: controller::SortDir) -> Self {
        match value {
            controller::SortDir::ASC => SortDir::ASC,
            controller::SortDir::DESC => SortDir::DESC,
        }
    }
}

#[derive(Clone, Debug, Validate, Deserialize, ToSchema)]
pub struct MonitorConfListRequest {
    pub filter: MonitorConfListFilter,
}

#[derive(Clone, Debug, Validate, Deserialize, ToSchema)]
pub struct MonitorConfListFilter {
    pub device_id: i32,
}

impl From<MonitorConfListFilter> for controller::MonitorConfListFilter {
    fn from(value: MonitorConfListFilter) -> Self {
        Self {
            device_id: value.device_id,
        }
    }
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct MonitorConfListResponse {
    result: Vec<MonitorConfListEntry>,
}

impl From<Vec<controller::MonitorConf>> for MonitorConfListResponse {
    fn from(mut value: Vec<controller::MonitorConf>) -> Self {
        MonitorConfListResponse {
            result: value
                .drain(..)
                .map(|v| MonitorConfListEntry {
                    id: v.id,
                    device_id: v.device_id,
                    sensor: v.sensor,
                    typ: v.typ.into(),
                    config: v.config.into(),
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct MonitorConfListEntry {
    pub id: i32,
    pub device_id: i32,
    pub sensor: String,
    pub typ: MonitorType,
    pub config: MonitorTypeConf,
}
