use std::vec;

use chrono;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{types::Json, Column, FromRow, Row, TypeInfo};

use crate::controller as ctrl;
use crate::query::integration::isqlx as sq;
use crate::{
    arg_from_ty, ref_arg_type,
    tool::query_trait::{ColumnsTrait, ValuesTrait},
};
use macros::Table;

use crate::debug_from_display;
use thiserror::Error;

#[derive(sqlx::Type, Debug, PartialEq)]
#[sqlx(type_name = "device_init_state", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceInitState {
    Device,
    Sensors,
}

impl ToString for DeviceInitState {
    fn to_string(&self) -> String {
        match self {
            DeviceInitState::Device => "DEVICE".into(),
            DeviceInitState::Sensors => "SENSORS".into(),
        }
    }
}

impl From<&DeviceInitState> for ctrl::DeviceInitState {
    fn from(v: &DeviceInitState) -> Self {
        match v {
            DeviceInitState::Device => ctrl::DeviceInitState::Device,
            DeviceInitState::Sensors => ctrl::DeviceInitState::Sensors,
        }
    }
}

ref_arg_type!(DeviceInitState);
arg_from_ty!(DeviceInitState);

#[derive(FromRow, Table)]
pub struct Device {
    #[column]
    pub id: i32,
    #[column]
    pub name: String,
    #[column]
    pub display_name: String,
    #[column]
    pub module_dir: String,
    #[column]
    pub data_dir: String,
    #[column]
    pub init_state: DeviceInitState,
}

impl Device {
    pub fn table_name() -> String {
        "device".into()
    }
}

// TODO: macro for this trait
impl ValuesTrait for Device {
    fn values(self, b: &mut crate::query::integration::isqlx::StatementBuilder) {
        b.values(vec![
            self.id.into(),
            self.name.into(),
            self.display_name.into(),
            self.module_dir.into(),
            self.data_dir.into(),
            self.init_state.into(),
        ]);
    }
}

#[derive(FromRow, Table)]
pub struct DeviceSensor {
    #[column]
    pub device_id: i32,
    #[column]
    pub sensor_name: String,
    #[column]
    pub sensor_table_name: String,
}

impl DeviceSensor {
    pub fn table_name() -> String {
        "device_sensor".into()
    }
}

impl ValuesTrait for DeviceSensor {
    fn values(self, b: &mut crate::query::integration::isqlx::StatementBuilder) {
        b.values(vec![
            self.device_id.into(),
            self.sensor_name.into(),
            self.sensor_table_name.into(),
        ]);
    }
}

/// For retrieving device's sensors data types from `information_schema.columns`
#[derive(FromRow, Table)]
pub struct ColumnType {
    #[column]
    pub table_name: String,
    #[column]
    pub column_name: String,
    #[column]
    pub udt_name: String,
}

pub struct SensorData {
    pub name: String,
    pub data: SensorDataTypeValue,
}

impl From<SensorData> for ctrl::SensorData {
    fn from(v: SensorData) -> Self {
        ctrl::SensorData {
            name: v.name,
            data: ctrl::SensorDataTypeValue::from(v.data),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SensorDataTypeValue {
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    Timestamp(chrono::NaiveDateTime),
    String(String),
    JSON(String),
}

impl crate::query::integration::isqlx::ArgType for SensorDataTypeValue {
    fn bind<'q>(
        &'q self,
        q: sqlx::query::Query<
            'q,
            sqlx::postgres::Postgres,
            <sqlx::postgres::Postgres as sqlx::database::HasArguments<'q>>::Arguments,
        >,
    ) -> sqlx::query::Query<
        'q,
        sqlx::postgres::Postgres,
        <sqlx::postgres::Postgres as sqlx::database::HasArguments<'q>>::Arguments,
    > {
        match self {
            SensorDataTypeValue::Int16(v) => v.bind(q),
            SensorDataTypeValue::Int32(v) => v.bind(q),
            SensorDataTypeValue::Int64(v) => v.bind(q),
            SensorDataTypeValue::Float32(v) => v.bind(q),
            SensorDataTypeValue::Float64(v) => v.bind(q),
            SensorDataTypeValue::Timestamp(v) => v.bind(q),
            SensorDataTypeValue::String(v) => v.bind(q),
            SensorDataTypeValue::JSON(v) => v.bind(q),
        }
    }
}

arg_from_ty!(SensorDataTypeValue);

impl From<ctrl::SensorDataTypeValue> for SensorDataTypeValue {
    fn from(v: ctrl::SensorDataTypeValue) -> Self {
        match v {
            ctrl::SensorDataTypeValue::Int16(v) => SensorDataTypeValue::Int16(v),
            ctrl::SensorDataTypeValue::Int32(v) => SensorDataTypeValue::Int32(v),
            ctrl::SensorDataTypeValue::Int64(v) => SensorDataTypeValue::Int64(v),
            ctrl::SensorDataTypeValue::Float32(v) => SensorDataTypeValue::Float32(v),
            ctrl::SensorDataTypeValue::Float64(v) => SensorDataTypeValue::Float64(v),
            ctrl::SensorDataTypeValue::Timestamp(v) => SensorDataTypeValue::Timestamp(v),
            ctrl::SensorDataTypeValue::String(v) => SensorDataTypeValue::String(v),
            ctrl::SensorDataTypeValue::JSON(v) => SensorDataTypeValue::JSON(v),
        }
    }
}

impl From<SensorDataTypeValue> for ctrl::SensorDataTypeValue {
    fn from(v: SensorDataTypeValue) -> Self {
        match v {
            SensorDataTypeValue::Int16(v) => ctrl::SensorDataTypeValue::Int16(v),
            SensorDataTypeValue::Int32(v) => ctrl::SensorDataTypeValue::Int32(v),
            SensorDataTypeValue::Int64(v) => ctrl::SensorDataTypeValue::Int64(v),
            SensorDataTypeValue::Float32(v) => ctrl::SensorDataTypeValue::Float32(v),
            SensorDataTypeValue::Float64(v) => ctrl::SensorDataTypeValue::Float64(v),
            SensorDataTypeValue::Timestamp(v) => ctrl::SensorDataTypeValue::Timestamp(v),
            SensorDataTypeValue::String(v) => ctrl::SensorDataTypeValue::String(v),
            SensorDataTypeValue::JSON(v) => ctrl::SensorDataTypeValue::JSON(v),
        }
    }
}

impl From<ctrl::SensorDataTypeValue> for Box<SensorDataTypeValue> {
    fn from(v: ctrl::SensorDataTypeValue) -> Self {
        Box::new(SensorDataTypeValue::from(v))
    }
}

pub struct SensorDataRow(pub Vec<SensorData>);

#[derive(Error)]
pub enum SensorDataDecodeError {
    #[error("static_dir is empty")]
    UnsupportedType(String),
}

debug_from_display!(SensorDataDecodeError);

impl<'r> FromRow<'r, PgRow> for SensorDataRow {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let mut res = Vec::with_capacity(row.len());

        for col in row.columns() {
            let info = col.type_info();
            let data = match info.name() {
                "INT2" => Ok(SensorDataTypeValue::Int16(row.get(col.ordinal()))),
                "INT4" => Ok(SensorDataTypeValue::Int32(row.get(col.ordinal()))),
                "INT8" => Ok(SensorDataTypeValue::Int64(row.get(col.ordinal()))),
                "FLOAT4" => Ok(SensorDataTypeValue::Float32(row.get(col.ordinal()))),
                "FLOAT8" => Ok(SensorDataTypeValue::Float64(row.get(col.ordinal()))),
                "TIMESTAMP" => Ok(SensorDataTypeValue::Timestamp(row.get(col.ordinal()))),
                "TEXT" => Ok(SensorDataTypeValue::String(row.get(col.ordinal()))),
                "JSONB" => Ok(SensorDataTypeValue::JSON(row.get(col.ordinal()))),
                any => Err(sqlx::Error::ColumnDecode {
                    index: col.name().to_string(),
                    source: SensorDataDecodeError::UnsupportedType(any.to_string()).into(),
                }),
            }?;

            res.push(SensorData {
                name: col.name().to_string(),
                data,
            })
        }

        Ok(SensorDataRow(res))
    }
}

impl From<SensorDataRow> for ctrl::SensorDataList {
    fn from(mut value: SensorDataRow) -> Self {
        value
            .0
            .drain(..)
            .map(|v| ctrl::SensorData::from(v))
            .collect()
    }
}

#[derive(Default)]
pub struct SensorDataFilter {
    pub from: Option<(String, SensorDataTypeValue)>,
    pub to: Option<(String, SensorDataTypeValue)>,
    pub limit: Option<i32>,
    pub sort: Option<Sort>,
}

impl SensorDataFilter {
    pub fn apply(&self, b: &mut sq::StatementBuilder) {
        if let Some((ref col, ref val)) = self.from {
            b.whereq(sq::gt(col.clone(), val.clone()));
        }

        if let Some((ref col, ref val)) = self.to {
            b.whereq(sq::lt(col.clone(), val.clone()));
        }

        if let Some(ref v) = self.limit {
            b.limit(v.clone());
        }

        if let Some(ref v) = self.sort {
            v.apply(b);
        }
    }
}

impl From<ctrl::SensorDataFilter> for SensorDataFilter {
    fn from(v: ctrl::SensorDataFilter) -> Self {
        Self {
            from: v.from.map(|v| (v.0, SensorDataTypeValue::from(v.1))),
            to: v.to.map(|v| (v.0, SensorDataTypeValue::from(v.1))),
            limit: v.limit,
            sort: v.sort.map(|v| Sort::from(v)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SortDir {
    ASC,
    DESC,
}

impl ToString for SortDir {
    fn to_string(&self) -> String {
        match self {
            SortDir::ASC => "ASC".to_string(),
            SortDir::DESC => "DESC".to_string(),
        }
    }
}

impl From<ctrl::SortDir> for SortDir {
    fn from(v: ctrl::SortDir) -> Self {
        match v {
            ctrl::SortDir::ASC => SortDir::ASC,
            ctrl::SortDir::DESC => SortDir::DESC,
        }
    }
}

impl From<SortDir> for ctrl::SortDir {
    fn from(v: SortDir) -> Self {
        match v {
            SortDir::ASC => ctrl::SortDir::ASC,
            SortDir::DESC => ctrl::SortDir::DESC,
        }
    }
}

pub struct Sort {
    pub field: String,
    pub order: SortDir,
}

impl Sort {
    pub fn apply(&self, b: &mut sq::StatementBuilder) {
        b.order(self.field.clone() + " " + &self.order.to_string());
    }
}

impl From<ctrl::Sort> for Sort {
    fn from(v: ctrl::Sort) -> Self {
        Self {
            field: v.field,
            order: SortDir::from(v.order),
        }
    }
}

#[derive(FromRow, Table)]
pub struct MonitorConf {
    #[column]
    pub id: i32,
    #[column]
    pub device_id: i32,
    #[column]
    pub sensor: String,
    #[column]
    pub typ: MonitorType,
    #[column]
    pub config: Json<MonitorTypeConf>,
}

impl MonitorConf {
    pub fn table_name() -> String {
        "monitor_conf".into()
    }

    // TODO: improve `Table` macros for this case (id mustn't be included when inserting into this table)
    pub fn insert_columns() -> &'static [&'static str] {
        &["device_id", "sensor", "typ", "config"]
    }
}

impl From<ctrl::MonitorConf> for MonitorConf {
    fn from(v: ctrl::MonitorConf) -> Self {
        MonitorConf {
            id: v.id,
            device_id: v.device_id,
            sensor: v.sensor,
            typ: MonitorType::from(v.typ),
            config: Json(MonitorTypeConf::from(v.config)),
        }
    }
}

impl From<MonitorConf> for ctrl::MonitorConf {
    fn from(v: MonitorConf) -> Self {
        ctrl::MonitorConf {
            id: v.id,
            device_id: v.device_id,
            sensor: v.sensor,
            typ: ctrl::MonitorType::from(v.typ),
            config: ctrl::MonitorTypeConf::from(v.config.0),
        }
    }
}

ref_arg_type!(Json<MonitorTypeConf>);
arg_from_ty!(Json<MonitorTypeConf>);

impl ValuesTrait for MonitorConf {
    fn values(self, b: &mut crate::query::integration::isqlx::StatementBuilder) {
        b.values(vec![
            self.device_id.into(),
            self.sensor.into(),
            self.typ.into(),
            self.config.into(),
        ]);
    }
}

#[derive(sqlx::Type, Debug, PartialEq)]
#[sqlx(type_name = "monitor_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MonitorType {
    Log,
    Line,
}

ref_arg_type!(MonitorType);
arg_from_ty!(MonitorType);

impl From<ctrl::MonitorType> for MonitorType {
    fn from(v: ctrl::MonitorType) -> Self {
        match v {
            ctrl::MonitorType::Log => MonitorType::Log,
            ctrl::MonitorType::Line => MonitorType::Line,
        }
    }
}

impl From<MonitorType> for ctrl::MonitorType {
    fn from(v: MonitorType) -> Self {
        match v {
            MonitorType::Log => ctrl::MonitorType::Log,
            MonitorType::Line => ctrl::MonitorType::Line,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MonitorTypeConf {
    Log(MonitorLogConf),
    Line(MonitorLineConf),
}

impl From<ctrl::MonitorTypeConf> for MonitorTypeConf {
    fn from(v: ctrl::MonitorTypeConf) -> Self {
        match v {
            ctrl::MonitorTypeConf::Log(v) => MonitorTypeConf::Log(MonitorLogConf {
                fields: v.fields,
                sort_field: v.sort_field,
                sort_direction: SortDir::from(v.sort_direction),
                limit: v.limit,
            }),
            ctrl::MonitorTypeConf::Line(v) => MonitorTypeConf::Line(MonitorLineConf {
                x_field: v.x_field,
                y_field: v.y_field,
                limit: v.limit,
            }),
        }
    }
}

impl From<MonitorTypeConf> for ctrl::MonitorTypeConf {
    fn from(v: MonitorTypeConf) -> Self {
        match v {
            MonitorTypeConf::Log(v) => ctrl::MonitorTypeConf::Log(ctrl::MonitorLogConf {
                fields: v.fields,
                sort_field: v.sort_field,
                sort_direction: ctrl::SortDir::from(v.sort_direction),
                limit: v.limit,
            }),
            MonitorTypeConf::Line(v) => ctrl::MonitorTypeConf::Line(ctrl::MonitorLineConf {
                x_field: v.x_field,
                y_field: v.y_field,
                limit: v.limit,
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorLogConf {
    pub fields: Vec<String>,
    pub sort_field: String,
    pub sort_direction: SortDir,
    pub limit: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorLineConf {
    pub x_field: String,
    pub y_field: String,
    pub limit: i32,
}

pub struct MonitorConfListFilter {
    pub device_id: Option<i32>,
}

impl MonitorConfListFilter {
    pub fn apply(&self, b: &mut sq::StatementBuilder) {
        if let Some(ref device_id) = self.device_id {
            b.whereq(sq::eq("device_id".into(), device_id.clone()));
        }
    }
}

impl From<ctrl::MonitorConfListFilter> for MonitorConfListFilter {
    fn from(v: ctrl::MonitorConfListFilter) -> Self {
        Self {
            device_id: Some(v.device_id),
        }
    }
}
