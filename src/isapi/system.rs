use super::IsapiRequest;
use crate::error::Result;
use crate::models::DeviceInfo;

pub(crate) fn device_info_request() -> IsapiRequest {
    IsapiRequest::get("/ISAPI/System/deviceInfo")
}

pub(crate) fn parse_device_info(xml: &str) -> Result<DeviceInfo> {
    Ok(quick_xml::de::from_str(xml)?)
}
