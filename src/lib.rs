extern crate serde;
#[macro_use]
extern crate lazy_static;

use serde_derive::{Deserialize, Serialize};

pub const ARCH: &'static str = std::env::consts::ARCH;
pub const FAMILY: &'static str = std::env::consts::FAMILY;
pub const OS: &'static str = std::env::consts::OS;

lazy_static! {
    pub static ref BUILD_TIME: std::time::SystemTime = std::time::SystemTime::now();
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValXorIfThenElse<V: Default, W> {
    Val(V),
    IfThenElse {
        #[serde(rename = "if")]
        if_field: String,
        then: V,
        #[serde(rename = "else")]
        else_field: Option<W>,
    },
}

impl<V: Default, W> Default for ValXorIfThenElse<V, W> {
    fn default() -> Self {
        ValXorIfThenElse::Val(V::default())
    }
}

/* union StringOrT<T> {
    string: std::mem::ManuallyDrop<String>,
    t: std::mem::ManuallyDrop<T>
} */

#[derive(Debug, PartialEq, Eq)]
pub struct ParseValXorIfThenElseError;

impl std::str::FromStr for ValXorIfThenElse<String, String> {
    type Err = ParseValXorIfThenElseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ValXorIfThenElse::<String, String>::Val(s.into()))
    }
}

impl From<&str> for ValXorIfThenElse<String, String> {
    fn from(s: &str) -> Self {
        ValXorIfThenElse::<String, String>::Val(s.into())
    }
}
impl<V: Default, W> From<V> for ValXorIfThenElse<V, W> {
    fn from(t: V) -> Self {
        ValXorIfThenElse::<V, _>::Val(t)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    #[serde(default = "default_name")]
    pub name: ValXorIfThenElse<std::borrow::Cow<'static, str>, std::borrow::Cow<'static, str>>,
    pub version: Option<ValXorIfThenElse<String, String>>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub repo: Option<String>,
    pub authors: Vec<String>,

    pub stack: indexmap::IndexMap<String, Vec<ServerConfiguration>>,
    pub stack_state: indexmap::IndexMap<String, StateValues>,
    pub stack_routing: Vec<ProtocolConfiguration>,

    pub component: Vec<Component>,
    /// environment variables. Priority: `ServerConfiguration` | `Component`; `Root`; system.
    pub env_vars: Option<indexmap::IndexMap<String, String>>,
}

const fn default_name(
) -> ValXorIfThenElse<std::borrow::Cow<'static, str>, std::borrow::Cow<'static, str>> {
    const NAME: &'static str = "verman-root";
    ValXorIfThenElse::Val(std::borrow::Cow::Borrowed(NAME))
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateValues {
    pub kind: Option<ValXorIfThenElse<String, String>>,
    pub install: Option<ValXorIfThenElse<State, State>>,
    pub remove: Option<ValXorIfThenElse<State, State>>,
    pub start: Option<ValXorIfThenElse<State, State>>,
    pub stop: Option<ValXorIfThenElse<State, State>>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// [app/component] install or reinstall
    /// [service] start or restart
    Always,

    /// [app/component] install if not installed
    /// [service] reload if stopped otherwise start
    Graceful,

    /// [app/component] use if installed otherwise move to next
    /// - error if no app/component of `kind` is found
    /// [service] use if `ping`able otherwise move to next
    /// - error if no service of `kind` iss pingable
    /// - otherwise set env var | config for pingable service
    #[default]
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
    /// environment variables. Priority: `ServerConfiguration` | `component`; `Root`; system.
    pub env_vars: Option<indexmap::IndexMap<String, String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolConfiguration {
    /// E.g., "localhost" | "127.0.0.1" | "::1" | "my_name.verman.io"
    pub name: Option<ValXorIfThenElse<String, String>>,

    /// E.g., "https" | "http"
    pub protocol: Option<ValXorIfThenElse<String, String>>,

    /// E.g., "LetsEncrypt"
    pub certificate_vendor: Option<ValXorIfThenElse<String, String>>,
}

/// URI generalised to UTF8 https://en.wikipedia.org/wiki/Internationalized_Resource_Identifier
// type Iri = String;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Component {
    pub src_uri: Option<ValXorIfThenElse<String, String>>,
    pub dst_uri: Option<ValXorIfThenElse<String, String>>,
    pub constraints: Vec<Constraint>,
    /// environment variables. Priority: `ServerConfiguration` | `Component`; `Root`; system.
    pub env_vars: Option<indexmap::IndexMap<String, String>>,
    pub mounts: Option<Vec<Mount>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Mount {
    pub when: String,
    pub uri: Option<ValXorIfThenElse<String, String>>,
    pub src_uri: Option<ValXorIfThenElse<String, String>>,
    pub action: String,
    // Future: enable an IDL (like JSON-schema) to validate these args
    // Future: allow this IDL to be provided by URI
    pub action_args: Option<serde_json::Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Constraint {
    pub kind: String,
    pub required_variant: Option<String>,
    pub required_version: Option<String>,
}

/// OSs from https://github.com/rust-lang/rust/blob/1.77.0/library/std/src/env.rs#L947-L961
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
    #[default]
    Unspecified,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorVersion {
    pub vendor: Option<ValXorIfThenElse<String, String>>,
    pub version: Option<ValXorIfThenElse<String, String>>,
}

/* pub fn eval_field(s: Box<dyn Into<String>>) -> String {
    s.into()
} */

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const VERMAN_JSON: &'static str = include_str!("verman.json");
    const VERMAN_TOML: &'static str = include_str!("verman.toml");

    #[test]
    fn it_serdes() {
        let config = Root {
            name: std::borrow::Cow::from(env!("CARGO_PKG_NAME")).into(),
            version: Some(String::from(env!("CARGO_PKG_VERSION")).into()),
            license: Some(String::from("(Apache-2.0 OR MIT)")),
            homepage: Some(String::from("https://verman.io")),
            repo: Some(String::from("https://github.com/verman-io")),
            authors: vec![String::from(env!("CARGO_PKG_AUTHORS"))],

            stack_state: {
                let mut state = indexmap::IndexMap::<String, StateValues>::new();
                state.insert(
                    String::from("database"),
                    StateValues {
                        kind: Some(String::from("sql").into()),
                        install: Some(State::Always.into()),
                        remove: None,
                        start: Some(State::Always.into()),
                        stop: None,
                    },
                );
                state.insert(
                    String::from("application_server"),
                    StateValues {
                        kind: None,
                        install: Some(State::Always.into()),
                        remove: None,
                        start: Some(State::Always.into()),
                        stop: None,
                    },
                );
                state.insert(
                    String::from("routing"),
                    StateValues {
                        kind: None,
                        install: Some(State::Always.into()),
                        remove: None,
                        start: Some(State::Always.into()),
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
                        env_vars: None,
                    }],
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
                            env_vars: None,
                        },
                        ServerConfiguration {
                            kind: String::from("ruby"),
                            versions: None,
                            server_priority: None,
                            env_vars: None,
                        },
                    ],
                );
                stack
            },
            stack_routing: vec![ProtocolConfiguration {
                name: Some(String::from("my_name.verman.io").into()),
                protocol: Some(String::from("https").into()),
                certificate_vendor: Some(String::from("LetsEncrypt").into()),
            }],
            component: vec![
                Component {
                    src_uri: Some(String::from("file://python_api_folder/").into()),
                    dst_uri: Some(String::from("http://localhost:${env.PYTHON_API_PORT}").into()),
                    constraints: vec![
                        Constraint {
                            kind: String::from("lang"),
                            required_variant: Some(String::from("python")),
                            required_version: None,
                        },
                        Constraint {
                            kind: String::from("OS"),
                            required_variant: None,
                            required_version: None,
                        },
                    ],
                    env_vars: None,
                    mounts: None,
                },
                Component {
                    src_uri: Some(String::from("file://ruby_api_folder/").into()),
                    dst_uri: Some(ValXorIfThenElse::IfThenElse {
                        if_field: String::from("OS == \"windows\""),
                        then: String::from("\"\\\\.\\pipe\\PipeName\""),
                        else_field: Some(String::from("\"unix:///var/run/my-socket.sock\"")),
                    }),
                    constraints: vec![
                        Constraint {
                            kind: String::from("lang"),
                            required_variant: Some(String::from("ruby")),
                            required_version: Some(String::from(">3.1.2, <3.2")),
                        },
                        Constraint {
                            kind: String::from("OS"),
                            required_variant: Some(String::from("${\"linux\" || \"windows\"}")),
                            required_version: None,
                        },
                    ],
                    env_vars: None,
                    mounts: None,
                },
                Component {
                    src_uri: None,
                    dst_uri: Some(String::from("my_app.verman.io").into()),
                    mounts: Some(vec![
                        Mount {
                            when: String::from("OS == \"windows\""),
                            uri: Some(String::from("file://win_nginx.conf").into()),
                            src_uri: None,
                            action: String::from("nginx::make_site_available"),
                            action_args: Some(json!({ "upsert": true })),
                        },
                        Mount {
                            when: String::from(
                                "NOT EXISTS(ISPREFIXOF(\"nginx::\", ${.mounts[].action}))",
                            ),
                            uri: Some(String::from("/api/py").into()),
                            src_uri: Some(String::from("#!/jq\n.component[] | select(.constraints[] | .kind == \"lang\" and .required_variant == \"python\").dst_uri").into()),
                            action: String::from("mount::expose"),
                            action_args: None,
                        },
                        Mount {
                            when: String::from(
                                "NOT EXISTS(ISPREFIXOF(\"nginx::\", ${.mounts[].action}))",
                            ),
                            uri: Some(String::from("/api/ruby").into()),
                            src_uri: Some(String::from("#!/jq\n'.component[] | select(.constraints[] | .kind == \"lang\" and .required_variant == \"ruby\").dst_uri").into()),
                            action: String::from("mount::expose"),
                            action_args: None,
                        },
                        Mount {
                            when: String::from("BUILD_TIME > 2024"),
                            uri: Some(String::from("/api/demo").into()),
                            src_uri: None, // 404
                            action: String::from("mount::expose"),
                            action_args: None,
                        }
                    ]),
                    constraints: vec![Constraint {
                        kind: String::from("routing"),
                        required_variant: None,
                        required_version: None,
                    }],
                    env_vars: {
                        let mut env = indexmap::IndexMap::<String, String>::new();
                        env.insert(
                            String::from("COMPONENT_NAME"),
                            String::from("mount_component"),
                        );
                        Some(env)
                    },
                },
            ],
            env_vars: {
                let mut env = indexmap::IndexMap::<String, String>::new();
                env.insert(String::from("DEBUG_ROOT"), String::from("true"));
                Some(env)
            },
        };
        // std::fs::write("./src/verman.json", serde_json::to_string(&config).unwrap()).unwrap();
        let root_from_json: Root = serde_json::from_str(&VERMAN_JSON).unwrap();
        // std::fs::write("./src/verman.toml", toml::to_string(&config).unwrap()).unwrap();
        let root_from_toml: Root = toml::from_str(&VERMAN_TOML).unwrap();
        assert_eq!(root_from_toml, root_from_json);
        assert_eq!(
            serde_json::to_string(&config).unwrap(),
            serde_json::to_string(&root_from_json).unwrap()
        );
        assert_eq!(
            toml::to_string(&config).unwrap(),
            toml::to_string(&root_from_toml).unwrap()
        );
    }
}
