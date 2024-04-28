extern crate serde;

use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub name: String,
    pub version: String,
    pub license: String,
    pub homepage: String,
    pub repo: String,
    pub authors: Vec<String>,

    pub stack: Stack,
    pub component: Vec<Component>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stack {
    pub state: std::collections::HashMap<String, StateValues>,
    pub database: Vec<ServerConfiguration>,
    #[serde(rename = "application_server")]
    pub application_server: Vec<ServerConfiguration>,
    #[serde(rename = "web_server_or_proxy")]
    pub web_server_or_proxy: Vec<ProtocolConfiguration>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateValues {
    #[serde(rename = "type")]
    pub type_field: String,
    pub install: Option<String>,
    pub remove: Option<String>,
    pub start: Option<String>,
    pub stop: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfiguration {
    #[serde(rename = "type")]
    pub type_field: String,
    pub versions: Option<Vec<String>>,
    #[serde(rename = "server_priority")]
    pub server_priority: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolConfiguration {
    /// E.g., "localhost" | "127.0.0.1" | "::1" | "my_name.verman.io"
    pub name: String,

    /// E.g., "https" | "http"
    pub protocol: String,

    /// E.g., "LetsEncrypt"
    pub certificate_vendor: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub src: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub version: Option<String>,
    pub expose: Value,
    pub config: Option<Config>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeAndLocation {
    #[serde(rename = "type")]
    pub type_field: String,

    pub location: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub mounts: std::collections::BTreeMap<String, TypeAndLocation>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mount {
    #[serde(rename = "/")]
    pub field: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serializes() {
        let config = Root {
            name: String::from(env!("CARGO_PKG_NAME")),
            version: String::from(env!("CARGO_PKG_VERSION")),
            license: String::from("(Apache-2.0 OR MIT)"),
            homepage: String::from("https://verman.io"),
            repo: String::from("https://github.com/verman-io"),
            authors: vec![String::from(env!("CARGO_PKG_AUTHORS"))],

            stack: Stack {
                state: {
                    let mut state = std::collections::HashMap::<String, StateValues>::new();
                    state.insert(
                        String::from("database"),
                        StateValues {
                            type_field: String::from("sql"),
                            install: Some(String::from("always")),
                            remove: None,
                            start: None,
                            stop: None,
                        },
                    );
                    state
                },
                database: vec![ServerConfiguration {
                    type_field: String::from("sql"),
                    versions: None,
                    server_priority: None,
                }],
                application_server: vec![],
                web_server_or_proxy: vec![],
            },
            component: vec![],
        };
        let j = serde_json::to_string(&config).unwrap();
        assert_eq!(
            j,
            r###"{"name":"verman-schema-rs","version":"0.1.0","license":"(Apache-2.0 OR MIT)","homepage":"https://verman.io","repo":"https://github.com/verman-io","authors":[""],"stack":{"state":{"database":{"type":"sql","install":"always","remove":null,"start":null,"stop":null}},"database":[{"type":"sql","versions":null,"server_priority":null}],"application_server":[],"web_server_or_proxy":[]},"component":[]}"###
        );
    }
}
