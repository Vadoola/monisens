use actix_multipart::form::MultipartForm;
use actix_web::{get, post, web, HttpResponse, Responder};
use actix_web_validator::Json;

use crate::webserver::model::{contract, ServiceState};

use super::super::model::error::WebError;

#[utoipa::path(
    context_path = "/service",
    request_body(content = DeviceStartInitRequest, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Ok response with device id and connection params", body = DeviceStartInitResponse),
        (status = "default", description = "Server error response", body = WebError),
    ),
)]
#[post("/start-device-init")]
pub async fn start_device_init(
    data: web::Data<ServiceState>,
    MultipartForm(form): MultipartForm<contract::DeviceStartInitRequest>,
) -> Result<impl Responder, WebError> {
    let mut file = tokio::fs::File::open(form.module_file.file.path())
        .await
        .map_err(|err| Box::<dyn std::error::Error>::from(err))?;

    let res = data
        .ctrl
        .start_device_init(form.device_name.to_string(), &mut file)
        .await?;

    Ok(web::Json(contract::DeviceStartInitResponse::from(res)))
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = ConnectDeviceRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ok response"),
        (status = "default", description = "Server error response", body = WebError),
    ),
)]
#[post("/connect-device")]
pub async fn connect_device(
    data: web::Data<ServiceState>,
    mut req: web::Json<contract::ConnectDeviceRequest>,
) -> Result<impl Responder, WebError> {
    data.ctrl.connect_device(
        req.device_id,
        req.connect_conf.drain(..).map(|v| v.into()).collect(),
    )?;

    Ok(HttpResponse::Ok())
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = ObtainDeviceConfInfoRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ok response with device conf info", body = ObtainDeviceConfInfoResponse),
        (status = "default", description = "Server error response", body = WebError),
    ),
)]
#[post("/obtain-device-conf-info")]
pub async fn obtain_device_conf_info(
    data: web::Data<ServiceState>,
    req: web::Json<contract::ObtainDeviceConfInfoRequest>,
) -> Result<impl Responder, WebError> {
    let mut res = data.ctrl.obtain_device_conf_info(req.device_id)?;

    Ok(web::Json(contract::ObtainDeviceConfInfoResponse {
        device_conf_info: res.drain(..).map(|v| v.into()).collect(),
    }))
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = ConfigureDeviceRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ok response"),
        (status = "default", description = "Server error response", body = WebError),
    ),
)]
#[post("/configure-device")]
pub async fn configure_device(
    data: web::Data<ServiceState>,
    mut req: web::Json<contract::ConfigureDeviceRequest>,
) -> Result<impl Responder, WebError> {
    data.ctrl
        .configure_device(
            req.device_id,
            req.confs.drain(..).map(|v| v.into()).collect(),
        )
        .await?;

    Ok(HttpResponse::Ok())
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = InterruptDeviceInitRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ok response"),
        (status = "default", description = "Server error response", body = WebError),
    ),
)]
#[post("/interrupt-device-init")]
pub async fn interrupt_device_init(
    data: web::Data<ServiceState>,
    req: web::Json<contract::InterruptDeviceInitRequest>,
) -> Result<impl Responder, WebError> {
    data.ctrl.interrupt_device_init(req.device_id).await?;

    Ok(HttpResponse::Ok())
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = GetSensorDataRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ok response", body = GetSensorDataResponse),
        (status = "default", description = "Server error response", body = WebError),
    ),
)]
#[post("/get-sensor-data")]
pub async fn get_sensor_data(
    data: web::Data<ServiceState>,
    req: Json<contract::GetSensorDataRequest>,
) -> Result<impl Responder, WebError> {
    let res = data.ctrl.get_sensor_data(req.0.clone().into()).await?;

    Ok(web::Json::<contract::GetSensorDataResponse>(res.into()))
}

#[utoipa::path(
    context_path = "/service",
    responses(
        (status = 200, description = "Ok response", body = GetDeviceListResponse),
        (status = "default", description = "Server error response", body = WebError),
    ),
)]
#[get("/get-device-list")]
pub async fn get_device_list(data: web::Data<ServiceState>) -> Result<impl Responder, WebError> {
    let mut res = data.ctrl.get_device_info_list()?;

    res.sort_unstable_by(|a, b| a.id.partial_cmp(&b.id).unwrap());

    Ok(web::Json::<contract::GetDeviceListResponse>(res.into()))
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = GetDeviceSensorInfoRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ok response with device conf info", body = GetDeviceSensorInfoResponse),
        (status = "default", description = "Server error response", body = WebError),
    ),
)]
#[post("/get-device-sensor-info")]
pub async fn get_device_sensor_info(
    data: web::Data<ServiceState>,
    req: Json<contract::GetDeviceSensorInfoRequest>,
) -> Result<impl Responder, WebError> {
    let res = data.ctrl.get_device_sensor_info(req.device_id)?;

    Ok(web::Json::<contract::GetDeviceSensorInfoResponse>(
        res.into(),
    ))
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = SaveMonitorConfRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ok response", body = SaveMonitorConfResponse),
        (status = "default", description = "Server error response", body = WebError),
    ),
)]
#[post("/save-monitor-conf")]
pub async fn save_monitor_conf(
    data: web::Data<ServiceState>,
    req: Json<contract::SaveMonitorConfRequest>,
) -> Result<impl Responder, WebError> {
    let id = data.ctrl.save_monitor_conf(req.0.into()).await?;

    Ok(web::Json(contract::SaveMonitorConfResponse { id }))
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = MonitorConfListRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ok response", body = MonitorConfListResponse),
        (status = "default", description = "Server error response", body = WebError),
    ),
)]
#[post("/get-monitor-conf-list")]
pub async fn get_monitor_conf_list(
    data: web::Data<ServiceState>,
    req: Json<contract::MonitorConfListRequest>,
) -> Result<impl Responder, WebError> {
    let res = data.ctrl.get_monitor_conf_list(req.0.filter.into()).await?;

    Ok(web::Json::<contract::MonitorConfListResponse>(res.into()))
}
