pub(crate) mod shared;

#[path = "echo/echo.rs"]
pub mod echo;

#[path = "env/env.rs"]
pub mod env;

#[path = "http_client/http_client.rs"]
pub mod http_client;

#[path = "interpolate/interpolate.rs"]
pub mod interpolate;

#[path = "jaq/jaq.rs"]
pub mod jaq;

#[path = "set_env/set_env.rs"]
pub mod set_env;

/*pub const VALID_COMMANDS_SET: std::collections::HashSet<&'static str> =
std::collections::HashSet::<&'static str>::from(VALID_COMMANDS);*/
pub mod command;
