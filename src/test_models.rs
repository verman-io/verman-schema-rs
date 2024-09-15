/// Whole file is protected with a `#[cfg(test)]` guard

pub const HTTPBIN_URL: &'static str = match option_env!("HTTPBIN_URL") {
    Some(url) => url,
    None => "https://httpbin.org",
};

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HttpBinPostResponse {
    pub args: serde_json::Value,
    pub data: String,
    pub files: serde_json::Value,
    pub form: serde_json::Value,
    pub headers: HttpBinPostHeaders,
    pub json: serde_json::Value,
    pub origin: String,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
// #[serde(deny_unknown_fields)]
pub(crate) struct HttpBinPostHeaders {
    #[serde(rename = "Accept")]
    pub accept: String,
    #[serde(rename = "Host")]
    pub host: String,

    #[serde(skip, rename = "X-Amzn-Trace-Id")]
    x_amzn_trace_id: String,
}

#[derive(Debug, PartialEq, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Message {
    pub(crate) message: String,
}
