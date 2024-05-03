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
    pub stack_routing: Vec<ProtocolConfiguration>,

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
    /* `vendor` example: {"nginx": {"windows": "./win_nginx.site_avail.conf",
    "_": "./nginx.site_avail.conf"}} */
    pub vendor: Option<indexmap::IndexMap<String, indexmap::IndexMap<usize, KindAndLocation>>>,
    pub mounts: Option<indexmap::IndexMap<String, KindAndLocation>>,
}

/// OSs from https://github.com/rust-lang/rust/blob/1.77.0/library/std/src/env.rs#L947-L961
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Os {
    Linux,
    Macos,
    Ios,
    Freebsd,
    Dragonfly,
    Netbsd,
    Openbsd,
    Solaris,
    Android,
    Windows,
    /// Sometimes useful for all OSs
    Unspecified,
}

impl Default for Os {
    fn default() -> Self {
        Os::Unspecified
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorVersion {
    pub vendor: String,
    pub version: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KindAndLocation {
    pub kind: String,
    pub location: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    const VERMAN_JSON: &'static str = include_str!("verman.json");

    #[test]
    fn it_serdes() {
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
                        String::from("routing"),
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
            stack_routing: vec![ProtocolConfiguration{
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
                    vendor: None,
                    mounts: None,
                },
                Component{
                    src: Some(String::from("./ruby_api_folder/")),
                    kind: String::from("ruby"),
                    version: Some(String::from(">3.1.2, <3.2")),
                    uri: String::from("${if(WIN32) { \"\\\\.\\pipe\\PipeName\" } else { \"unix:///var/run/my-socket.sock\" }}"),
                    vendor: None,
                    mounts: None,
                },
                Component{
                    src: None,
                    kind: String::from("routing"),
                    version: None,
                    uri: String::from("my_app.verman.io"),
                    vendor: {
                        let mut vendor = indexmap::IndexMap::<String, indexmap::IndexMap::<usize, KindAndLocation>>::new();
                        vendor.insert(String::from("nginx"), {
                            let mut os_to_kind_and_location = indexmap::IndexMap::<usize, KindAndLocation>::new();
                            os_to_kind_and_location.insert(Os::Windows as usize, KindAndLocation { kind: String::from("server_block"), location: String::from("./win_nginx.site_avail.conf") });
                            os_to_kind_and_location.insert(Os::Unspecified as usize, KindAndLocation { kind: String::from("server_block"), location: String::from("./nginx.site_avail.conf") });
                            os_to_kind_and_location
                        });
                        Some(vendor)
                    },
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
        let root: Root = serde_json::from_str(&VERMAN_JSON).unwrap();
        assert_eq!(
            serde_json::to_string(&config).unwrap(),
            serde_json::to_string(&root).unwrap()
        );
    }
}
