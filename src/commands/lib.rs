use crate::models::{CommonContent, HttpCommandArgs};

#[path = "echo/echo.rs"]
pub mod echo;

#[path = "http_client/http_client.rs"]
pub mod http_client;

#[path = "jaq/jaq.rs"]
pub mod jaq;

#[path = "set_env/set_env.rs"]
pub mod set_env;

/*pub const VALID_COMMANDS_SET: std::collections::HashSet<&'static str> =
std::collections::HashSet::<&'static str>::from(VALID_COMMANDS);*/

#[derive(
    Debug,
    Clone,
    /* strum_macros::EnumString,
    strum_macros::VariantNames, */
    serde_derive::Deserialize,
    serde_derive::Serialize,
)]
// #[strum(serialize_all = "snake_case")]
pub enum CommandName {
    Echo,
    HttpClient,
    SetEnv,
    Jaq,
}

#[derive(Debug, Clone, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(untagged)]
pub enum CommandArgs {
    Echo(CommonContent),
    HttpClient(HttpCommandArgs),
    SetEnv(CommonContent),
    Jaq(CommonContent),
}
