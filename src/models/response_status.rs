use std::fmt;

use serde::Deserialize;

/// The `<ResponseStatus>` document ISAPI returns for writes and for errors.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ResponseStatus {
    #[serde(rename = "requestURL", default)]
    pub request_url: Option<String>,

    /// 1 = OK, 2 = Device Busy, 3 = Device Error, 4 = Invalid Operation,
    /// 5 = Invalid XML Format, 6 = Invalid XML Content, 7 = Reboot Required.
    #[serde(rename = "statusCode", default)]
    pub status_code: Option<i32>,

    #[serde(rename = "statusString", default)]
    pub status_string: Option<String>,

    /// Detailed reason, e.g. `notSupport`, `badParameters`, `invalidID`.
    #[serde(rename = "subStatusCode", default)]
    pub sub_status_code: Option<String>,

    #[serde(rename = "errorCode", default)]
    pub error_code: Option<i64>,

    #[serde(rename = "errorMsg", default)]
    pub error_msg: Option<String>,
}

impl ResponseStatus {
    /// Parses an ISAPI `ResponseStatus` body. Returns `None` for bodies that
    /// are not a `ResponseStatus` (HTML error pages, empty bodies, ...).
    pub fn parse(body: &str) -> Option<Self> {
        let status: Self = quick_xml::de::from_str(body).ok()?;

        (status.status_code.is_some() || status.sub_status_code.is_some()).then_some(status)
    }

    /// `true` for "OK" (1) and "Reboot Required" (7, the change was applied).
    pub fn is_success(&self) -> bool {
        matches!(self.status_code, None | Some(1) | Some(7))
    }
}

impl fmt::Display for ResponseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        if let Some(code) = self.status_code {
            match &self.status_string {
                Some(text) => parts.push(format!("{} ({})", text, code)),
                None => parts.push(format!("statusCode {}", code)),
            }
        }
        if let Some(sub) = &self.sub_status_code {
            parts.push(sub.clone());
        }
        if let Some(msg) = &self.error_msg {
            parts.push(msg.clone());
        }

        write!(f, "{}", parts.join(", "))
    }
}
