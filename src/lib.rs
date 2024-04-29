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

    pub stack: indexmap::IndexMap<String, Vec<ServerConfiguration>>,
    pub stack_state: indexmap::IndexMap<String, StateValues>,
    pub stack_web_server_or_proxy: Vec<ProtocolConfiguration>,

    pub component: Vec<Component>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateValues {
    pub kind: Option<String>,
    pub install: Option<State>,
    pub remove: Option<State>,
    pub start: Option<State>,
    pub stop: Option<State>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// [app/component] install or reinstall
    /// [service] start or restart
    Always,

    /// [app/component] install if not installed
    /// [service] reload if stopped otherwise start
    Graceful,

    /// [service] ping: if not started/installed:
    /// - ping next service in array until end
    /// - error if no services are pingable
    /// - otherwise set env var | config for pingable service
    Untouched,

    /// [service] stop service (if service is running)
    Stop,

    /// [service] uninstall service (if installed)
    /// [app/component] uninstall (if installed)
    Remove,

    /// [service] list which services would be started
    /// [app/component] list what would be installed* (*without making any network requests)
    DryRun,
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

            stack_state:  {
                    let mut state = indexmap::IndexMap::<String, StateValues>::new();
                    state.insert(
                        String::from("database"),
                        StateValues {
                            kind: Some(String::from("sql")),
                            install: Some(State::Always),
                            remove: None,
                            start: Some(State::Always),
                            stop: None,
                        },
                    );
                    state.insert(
                        String::from("application_server"),
                        StateValues {
                            kind: None,
                            install: Some(State::Always),
                            remove: None,
                            start: Some(State::Always),
                            stop: None,
                        },
                    );
                    state.insert(
                        String::from("web_server_or_proxy"),
                        StateValues {
                            kind: None,
                            install: Some(State::Always),
                            remove: None,
                            start: Some(State::Always),
                            stop: None,
                        },
                    );
                    state
            },

            stack: {
              let mut stack = indexmap::IndexMap::<String, Vec<ServerConfiguration>>::new();
                stack.insert(
                   String::from("database"),
                   vec![ServerConfiguration {
                       kind: String::from("sql"),
                       versions: None,
                       server_priority: None,
                   }]
                );
                stack.insert(
                    String::from("application_server"),
                    vec![
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
                    ]
                );
                stack
            },
            stack_web_server_or_proxy: vec![ProtocolConfiguration{
                name: String::from("my_name.verman.io"),
                protocol: String::from("https"),
                certificate_vendor: Some(String::from("LetsEncrypt")),
            }],
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
                                location: String::from("${stack.components[.kind==\"python\"].uri}")
                            }
                        );
                        mounts.insert(
                            String::from("/api/ruby"),
                            KindAndLocation {
                                kind: String::from("ruby"),
                                location: String::from("${stack.components[.kind==\"ruby\"].uri}")
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
            r###"{"name":"verman-schema-rs","version":"0.0.1","license":"(Apache-2.0 OR MIT)","homepage":"https://verman.io","repo":"https://github.com/verman-io","authors":[""],"stack":{"database":[{"kind":"sql","versions":null,"server_priority":null}],"application_server":[{"kind":"python","versions":["~2.7","~3.6","~3.13"],"server_priority":["Waitress","mod_wsgi","uvicorn"]},{"kind":"ruby","versions":null,"server_priority":null}]},"stack_state":{"database":{"kind":"sql","install":"always","remove":null,"start":"always","stop":null},"application_server":{"kind":null,"install":"always","remove":null,"start":"always","stop":null},"web_server_or_proxy":{"kind":null,"install":"always","remove":null,"start":"always","stop":null}},"stack_web_server_or_proxy":[{"name":"my_name.verman.io","protocol":"https","certificate_vendor":"LetsEncrypt"}],"component":[{"src":"./python_api_folder/","kind":"python","version":">3.8","uri":"http://localhost:${env.PYTHON_API_PORT}","mounts":null},{"src":"./ruby_api_folder/","kind":"ruby","version":">3.1.2, <3.2","uri":"${if(WIN32) { \"\\\\.\\pipe\\PipeName\" } else { \"unix:///var/run/my-socket.sock\" }}","mounts":null},{"src":null,"kind":"web_server_or_proxy","version":null,"uri":"my_app.verman.io","mounts":{"/api/py":{"kind":"python","location":"${stack.components[.kind==\"python\"].uri}"},"/api/ruby":{"kind":"ruby","location":"${stack.components[.kind==\"ruby\"].uri}"},"/":{"kind":"static","location":"${env.WWWROOT}"}}}]}"###
        );
    }
}
