extern crate serde;

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
pub struct Stack {
    pub state: indexmap::IndexMap<String, StateValues>,
    pub database: Vec<ServerConfiguration>,
    pub application_server: Vec<ServerConfiguration>,
    pub web_server_or_proxy: Vec<ProtocolConfiguration>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateValues {
    pub kind: Option<String>,
    pub install: Option<String>,
    pub remove: Option<String>,
    pub start: Option<String>,
    pub stop: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerConfiguration {
    pub kind: String,
    pub versions: Option<Vec<String>>,
    pub server_priority: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolConfiguration {
    /// E.g., "localhost" | "127.0.0.1" | "::1" | "my_name.verman.io"
    pub name: String,

    /// E.g., "https" | "http"
    pub protocol: String,

    /// E.g., "LetsEncrypt"
    pub certificate_vendor: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    pub src: Option<String>,
    pub kind: String,
    pub version: Option<String>,
    pub uri: String,
    pub mounts: Option<indexmap::IndexMap<String, KindAndLocation>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KindAndLocation {
    pub kind: String,
    pub location: String,
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
                    let mut state = indexmap::IndexMap::<String, StateValues>::new();
                    state.insert(
                        String::from("database"),
                        StateValues {
                            kind: Some(String::from("sql")),
                            install: Some(String::from("always")),
                            remove: None,
                            start: Some(String::from("always")),
                            stop: None,
                        },
                    );
                    state.insert(
                        String::from("application_server"),
                        StateValues {
                            kind: None,
                            install: Some(String::from("always")),
                            remove: None,
                            start: Some(String::from("always")),
                            stop: None,
                        },
                    );
                    state.insert(
                        String::from("web_server_or_proxy"),
                        StateValues {
                            kind: None,
                            install: Some(String::from("always")),
                            remove: None,
                            start: Some(String::from("always")),
                            stop: None,
                        },
                    );
                    state
                },
                database: vec![ServerConfiguration {
                    kind: String::from("sql"),
                    versions: None,
                    server_priority: None,
                }],
                application_server: vec![
                    ServerConfiguration {
                        kind: String::from("python"),
                        versions: Some(vec![
                            String::from("~2.7"),
                            String::from("~3.6"),
                            String::from("~3.13"),
                        ]),
                        server_priority: Some(vec![
                            String::from("Waitress"),
                            String::from("mod_wsgi"),
                            String::from("uvicorn"),
                        ]),
                    },
                    ServerConfiguration {
                        kind: String::from("ruby"),
                        versions: None,
                        server_priority: None,
                    },
                ],
                web_server_or_proxy: vec![ProtocolConfiguration{
                    name: String::from("my_name.verman.io"),
                    protocol: String::from("https"),
                    certificate_vendor: Some(String::from("LetsEncrypt")),
                }],
            },
            component: vec![
                Component{
                    src: Some(String::from("./python_api_folder/")),
                    kind: String::from("python"),
                    version: Some(String::from(">3.8")),
                    uri: String::from("http://localhost:${env.PYTHON_API_PORT}"),
                    mounts: None,
                },
                Component{
                    src: Some(String::from("./ruby_api_folder/")),
                    kind: String::from("ruby"),
                    version: Some(String::from(">3.1.2, <3.2")),
                    uri: String::from("${if(WIN32) { \"\\\\.\\pipe\\PipeName\" } else { \"unix:///var/run/my-socket.sock\" }}"),
                    mounts: None,
                },
                Component{
                    src: None,
                    kind: String::from("web_server_or_proxy"),
                    version: None,
                    uri: String::from("my_app.verman.io"),
                    mounts: {
                        let mut mounts = indexmap::IndexMap::<String, KindAndLocation>::new();
                        mounts.insert(
                            String::from("/api/py"),
                            KindAndLocation {
                                kind: String::from("python"),
                                location: String::from("${stack.components[kind==\"python\"].uri}")
                            }
                        );
                        mounts.insert(
                            String::from("/api/ruby"),
                            KindAndLocation {
                                kind: String::from("ruby"),
                                location: String::from("${stack.components[kind==\"ruby\"].uri}")
                            }
                        );
                        mounts.insert(
                            String::from("/"),
                            KindAndLocation {
                                kind: String::from("static"),
                                location: String::from("${env.WWWROOT}")
                            }
                        );
                        Some(mounts)
                    },
                },
            ],
        };
        let j = serde_json::to_string(&config).unwrap();
        assert_eq!(
            j,
            r###"{"name":"verman-schema-rs","version":"0.0.1","license":"(Apache-2.0 OR MIT)","homepage":"https://verman.io","repo":"https://github.com/verman-io","authors":[""],"stack":{"state":{"database":{"kind":"sql","install":"always","remove":null,"start":"always","stop":null},"application_server":{"kind":null,"install":"always","remove":null,"start":"always","stop":null},"web_server_or_proxy":{"kind":null,"install":"always","remove":null,"start":"always","stop":null}},"database":[{"kind":"sql","versions":null,"server_priority":null}],"application_server":[{"kind":"python","versions":["~2.7","~3.6","~3.13"],"server_priority":["Waitress","mod_wsgi","uvicorn"]},{"kind":"ruby","versions":null,"server_priority":null}],"web_server_or_proxy":[{"name":"my_name.verman.io","protocol":"https","certificate_vendor":"LetsEncrypt"}]},"component":[{"src":"./python_api_folder/","kind":"python","version":">3.8","uri":"http://localhost:${env.PYTHON_API_PORT}","mounts":null},{"src":"./ruby_api_folder/","kind":"ruby","version":">3.1.2, <3.2","uri":"${if(WIN32) { \"\\\\.\\pipe\\PipeName\" } else { \"unix:///var/run/my-socket.sock\" }}","mounts":null},{"src":null,"kind":"web_server_or_proxy","version":null,"uri":"my_app.verman.io","mounts":{"/api/py":{"kind":"python","location":"${stack.components[kind==\"python\"].uri}"},"/api/ruby":{"kind":"ruby","location":"${stack.components[kind==\"ruby\"].uri}"},"/":{"kind":"static","location":"${env.WWWROOT}"}}}]}"###
        );
    }
}
