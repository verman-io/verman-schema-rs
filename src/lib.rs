extern crate serde;

use serde_derive::{Deserialize, Serialize};
use std::fmt;

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for ValXorIfThenElse<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        struct StringOrStruct<T>(std::marker::PhantomData<fn() -> T>);

        impl<'de, T: serde::Deserialize<'de>> serde::de::Visitor<'de> for StringOrStruct<T> {
            type Value = ValXorIfThenElse<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string or structure")
            }

            fn visit_str<E>(self, value: &str) -> Result<ValXorIfThenElse<T>, E>
                where
                    E: serde::de::Error,
            {
                Ok(ValXorIfThenElse::Val(value.to_owned()))
            }

            fn visit_map<A>(self, map: A) -> Result<ValXorIfThenElse<T>, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
            {
                Ok(ValXorIfThenElse::IfThenElse(serde::Deserialize::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?))
            }
        }

        deserializer.deserialize_any(StringOrStruct(std::marker::PhantomData))
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deserialize_with = "des_val_xor_if_then_else")]
pub enum ValXorIfThenElse<T> {
    Val(T),
    IfThenElse {
        #[serde(rename = "if")]
        if_field: String,
        then: T,
        #[serde(rename = "else")]
        else_field: Option<T>,
    },
}

union StringOrT<T> {
    string: std::mem::ManuallyDrop<String>,
    t: std::mem::ManuallyDrop<T>
}

impl Default for ValXorIfThenElse<String> {
    fn default() -> Self {
        ValXorIfThenElse::<String>::IfThenElse {
            if_field: String::from("true"), then: String::from("true"), else_field: None
        }
    }
}

/*impl <T>Into<ValXorIfThenElse<std::string::String>> for ValXorIfThenElse<T> {
    fn into(self) -> ValXorIfThenElse<String> {
        ValXorIfThenElse::Val(self)
    }
}*/

#[derive(Debug, PartialEq, Eq)]
pub struct ParseValXorIfThenElseError;

impl std::str::FromStr for ValXorIfThenElse<String> {
    type Err = ParseValXorIfThenElseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ValXorIfThenElse::<String>::Val(s.into()))
    }
}

impl From<&str> for ValXorIfThenElse<String> {
    fn from(s: &str) -> Self {
        ValXorIfThenElse::<String>::Val(s.into())
    }
}
impl <T>From<T> for ValXorIfThenElse<T> {
    fn from(t: T) -> Self {
        ValXorIfThenElse::<T>::Val(t)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    pub name: ValXorIfThenElse<String>,
    pub version: ValXorIfThenElse<String>,
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
    pub kind: Option<ValXorIfThenElse<String>>,
    pub install: Option<ValXorIfThenElse<State>>,
    pub remove: Option<ValXorIfThenElse<State>>,
    pub start: Option<ValXorIfThenElse<State>>,
    pub stop: Option<ValXorIfThenElse<State>>,
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

    /// [app/component] use if installed otherwise move to next
    /// - error if no app/component of `kind` is found
    /// [service] use if `ping`able otherwise move to next
    /// - error if no service of `kind` iss pingable
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
    pub name: ValXorIfThenElse<String>,

    /// E.g., "https" | "http"
    pub protocol: ValXorIfThenElse<String>,

    /// E.g., "LetsEncrypt"
    pub certificate_vendor: Option<ValXorIfThenElse<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    pub src: Option<ValXorIfThenElse<String>>,
    pub kind: ValXorIfThenElse<String>,
    pub version: Option<ValXorIfThenElse<String>>,
    pub uri: ValXorIfThenElse<String>,
    /* `vendor` example: {"nginx": {"windows": "./win_nginx.site_avail.conf",
    "_": "./nginx.site_avail.conf"}} */
    pub vendor: Option<indexmap::IndexMap<String, indexmap::IndexMap<Os, KindAndUri>>>,
    pub mounts: Option<indexmap::IndexMap<String, KindAndUri>>,
}

/// OSs from https://github.com/rust-lang/rust/blob/1.77.0/library/std/src/env.rs#L947-L961
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    Unspecified,
}

impl Default for Os {
    fn default() -> Self {
        Os::Unspecified
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorVersion {
    pub vendor: ValXorIfThenElse<String>,
    pub version: Option<ValXorIfThenElse<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KindAndUri {
    pub kind: ValXorIfThenElse<String>,
    pub uri: ValXorIfThenElse<String>,
}

/* pub fn eval_field(s: Box<dyn Into<String>>) -> String {
    s.into()
} */

#[cfg(test)]
mod tests {
    use super::*;

    const VERMAN_JSON: &'static str = include_str!("verman.json");
    const VERMAN_TOML: &'static str = include_str!("verman.toml");

    #[test]
    fn it_serdes() {
        let config = Root {
            name: String::from(env!("CARGO_PKG_NAME")).into(),
            version: String::from(env!("CARGO_PKG_VERSION")).into(),
            license: String::from("(Apache-2.0 OR MIT)"),
            homepage: String::from("https://verman.io"),
            repo: String::from("https://github.com/verman-io"),
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
                        },
                        ServerConfiguration {
                            kind: String::from("ruby"),
                            versions: None,
                            server_priority: None,
                        },
                    ],
                );
                stack
            },
            stack_routing: vec![ProtocolConfiguration {
                name: String::from("my_name.verman.io").into(),
                protocol: String::from("https").into(),
                certificate_vendor: Some(String::from("LetsEncrypt").into()),
            }],
            component: vec![
                Component {
                    src: Some(String::from("./python_api_folder/").into()),
                    kind: String::from("python").into(),
                    version: Some(String::from(">3.8").into()),
                    uri: String::from("http://localhost:${env.PYTHON_API_PORT}").into(),
                    vendor: None,
                    mounts: None,
                },
                Component {
                    src: Some(String::from("./ruby_api_folder/").into()),
                    kind: String::from("ruby").into(),
                    version: Some(String::from(">3.1.2, <3.2").into()),
                    uri: String::from("${if(WIN32) { \"\\\\.\\pipe\\PipeName\" } else { \"unix:///var/run/my-socket.sock\" }}").into(),
                    vendor: None,
                    mounts: None,
                },
                Component {
                    src: None,
                    kind: String::from("routing").into(),
                    version: None,
                    uri: String::from("my_app.verman.io").into(),
                    vendor: {
                        let mut vendor = indexmap::IndexMap::<String, indexmap::IndexMap::<Os, KindAndUri>>::new();
                        vendor.insert(String::from("nginx"), {
                            let mut os_to_kind_and_location = indexmap::IndexMap::<Os, KindAndUri>::new();
                            os_to_kind_and_location.insert(Os::Windows, KindAndUri { kind: String::from("server_block").into(), uri: String::from("./win_nginx.site_avail.conf").into() });
                            os_to_kind_and_location.insert(Os::Linux, KindAndUri { kind: String::from("server_block").into(), uri: String::from("./nginx.site_avail.conf").into() });
                            os_to_kind_and_location
                        });
                        Some(vendor)
                    },
                    mounts: {
                        let mut mounts = indexmap::IndexMap::<String, KindAndUri>::new();
                        mounts.insert(
                            String::from("/api/py"),
                            KindAndUri {
                                kind: String::from("python").into(),
                                uri: String::from("${stack.components[.kind==\"python\"].uri}").into(),
                            },
                        );
                        mounts.insert(
                            String::from("/api/ruby"),
                            KindAndUri {
                                kind: String::from("ruby").into(),
                                uri: String::from("${stack.components[.kind==\"ruby\"].uri}").into(),
                            },
                        );
                        mounts.insert(
                            String::from("/"),
                            KindAndUri {
                                kind: String::from("static").into(),
                                uri: String::from("file://${env.WWWROOT}").into(),
                            },
                        );
                        Some(mounts)
                    },
                },
            ],
        };
        let root_from_json: Root = serde_json::from_str(&VERMAN_JSON).unwrap();
        let root_from_toml: Root = toml::from_str(&VERMAN_TOML).unwrap();
        /* std::fs::write("./src/verman.toml", toml::to_string(&config).unwrap())
        .expect("Could not write to file!"); */
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
