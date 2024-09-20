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

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "cmd")]
pub enum CommandArgs {
    Echo(CommonContent),
    HttpClient(HttpCommandArgs),
    SetEnv(CommonContent),
    Jaq(CommonContent),
}

impl Default for CommandArgs {
    fn default() -> Self {
        Self::Echo(CommonContent::default())
    }
}
