#![feature(iter_collect_into)]
extern crate jaq_core;
extern crate jaq_interpret;
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    #[serde(default = "default_name")]
    pub name: std::borrow::Cow<'static, str>,
    pub version: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub repo: Option<String>,
    pub authors: Vec<String>,

    pub verman: VermanConfig,

    pub stack: indexmap::IndexMap<String, Vec<ServerConfiguration>>,
    pub stack_state: indexmap::IndexMap<String, StateValues>,
    pub stack_routing: Vec<ProtocolConfiguration>,

    pub component: Vec<Component>,
    /// environment variables. Priority: `ServerConfiguration` | `Component`; `Root`; system.
    pub env_vars: Option<indexmap::IndexMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VermanConfig {
    /// If shebang is provided, takes priority
    /// Special lines:
    /// - `"#!/jq"` uses internal `jaq` dependency (no external `jq`)
    /// - `"#!/echo` simply outputs the lines below it
    shell: String,
}

impl Default for VermanConfig {
    fn default() -> Self {
        Self {
            shell: String::from("#!/jq"),
        }
    }
}

const fn default_name() -> std::borrow::Cow<'static, str> {
    const NAME: &'static str = "verman-root";
    std::borrow::Cow::Borrowed(NAME)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateValues {
    pub kind: Option<String>,
    pub install: Option<State>,
    pub remove: Option<State>,
    pub start: Option<State>,
    pub stop: Option<State>,
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
    pub name: Option<String>,

    /// E.g., "https" | "http"
    pub protocol: Option<String>,

    /// E.g., "LetsEncrypt"
    pub certificate_vendor: Option<String>,
}

/// URI generalised to UTF8 https://en.wikipedia.org/wiki/Internationalized_Resource_Identifier
// type Iri = String;
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Component {
    pub src_uri: Option<String>,
    pub dst_uri: Option<String>,
    pub constraints: Vec<Constraint>,
    /// environment variables. Priority: `ServerConfiguration` | `Component`; `Root`; system.
    pub env_vars: Option<indexmap::IndexMap<String, String>>,
    pub mounts: Option<Vec<Mount>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Mount {
    pub when: String,
    pub uri: Option<String>,
    pub src_uri: Option<String>,
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
    pub vendor: Option<String>,
    pub version: Option<String>,
}

/* pub fn eval_field(s: Box<dyn Into<String>>) -> String {
    s
} */

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    Jaq(jaq_interpret::Error),
    /// Depressing
    UnexpectedEmptiness,
}

impl From<jaq_interpret::Error> for Error {
    fn from(err: jaq_interpret::Error) -> Self {
        Error::Jaq(err)
    }
}

fn jq<'a>(value: serde_json::Value, filter: &str) -> Result<String, Error> {
    // start out only from core filters,
    // which do not include filters in the standard library
    // such as `map`, `select` etc.
    let mut defs = jaq_interpret::ParseCtx::new(Vec::new());

    // parse the filter
    let (f, errs) = jaq_parse::parse(filter, jaq_parse::main());
    assert_eq!(errs, Vec::new());

    // compile the filter in the context of the given definitions
    let f = defs.compile(f.unwrap());
    assert!(defs.errs.is_empty());

    let inputs = jaq_interpret::RcIter::new(core::iter::empty());

    let out = jaq_interpret::FilterT::run(
        &f,
        (
            jaq_interpret::Ctx::new([], &inputs),
            <jaq_interpret::Val as From<serde_json::Value>>::from(value),
        ),
    );
    let mut results = Vec::<String>::new();
    out.filter_map(|val| Option::from(val.ok()?.to_string_or_clone()))
        .collect_into(&mut results);
    if results.is_empty() {
        Err(Error::UnexpectedEmptiness)
    } else {
        Ok(results.join("\n"))
    }
}

pub fn maybe_modify_string_via_shebang<'a, 'b>(
    vars: &'a indexmap::IndexMap<String, String>,
    s: &'b str,
) -> Result<std::borrow::Cow<'b, str>, Error> {
    if let Some((first_line, rest)) = s.split_once("\n") {
        if first_line.starts_with("#!/") {
            match first_line {
                "#!/jq" =>
                // `^ the `?` operator cannot be applied to type `Cow<'_, _>``
                {
                    match jq(serde_json::json!(vars["config"]), rest) {
                        Ok(jq_ified) => Ok(std::borrow::Cow::Owned(jq_ified)),
                        Err(e) => Err(e),
                    }
                }
                "#!/echo" => Ok(std::borrow::Cow::Borrowed(rest)),
                _ => unimplemented!("TODO: Generic shebang handling for: {}", first_line),
            }
        } else {
            Ok(std::borrow::Cow::Borrowed(s))
        }
    } else {
        Ok(std::borrow::Cow::Borrowed(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VERMAN_JSON: &'static str = include_str!("verman.json");
    const VERMAN_TOML: &'static str = include_str!("verman.toml");

    #[test]
    fn it_serdes() {
        let config = Root {
            name: std::borrow::Cow::from(env!("CARGO_PKG_NAME")),
            version: Some(String::from(env!("CARGO_PKG_VERSION"))),
            license: Some(String::from("(Apache-2.0 OR MIT)")),
            homepage: Some(String::from("https://verman.io")),
            repo: Some(String::from("https://github.com/verman-io")),
            authors: vec![String::from(env!("CARGO_PKG_AUTHORS"))],

            verman: VermanConfig::default(),

            stack_state: {
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
                name: Some(String::from("my_name.verman.io")),
                protocol: Some(String::from("https")),
                certificate_vendor: Some(String::from("LetsEncrypt")),
            }],
            component: vec![
                Component {
                    src_uri: Some(String::from("file://python_api_folder/")),
                    dst_uri: Some(String::from("http://localhost:${PYTHON_API_PORT}")),
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
                    src_uri: Some(String::from("file://ruby_api_folder/")),
                    dst_uri: Some(String::from("#!/jq\nif $OS == \"windows\" then \"\\\\.\\pipe\\PipeName\" else \"unix:///var/run/my-socket.sock\"")),
                    constraints: vec![
                        Constraint {
                            kind: String::from("lang"),
                            required_variant: Some(String::from("ruby")),
                            required_version: Some(String::from(">3.1.2, <3.2")),
                        },
                        Constraint {
                            kind: String::from("OS"),
                            required_variant: Some(String::from("#!/jq\n$OS | in({\"linux\" || \"windows\"})")),
                            required_version: None,
                        },
                    ],
                    env_vars: None,
                    mounts: None,
                },
                Component {
                    src_uri: None,
                    dst_uri: Some(String::from("my_app.verman.io")),
                    mounts: Some(vec![
                        Mount {
                            when: String::from("#!/jq\n$OS == \"windows\""),
                            uri: Some(String::from("file://win_nginx.conf")),
                            src_uri: None,
                            action: String::from("nginx::make_site_available"),
                            action_args: Some(serde_json::json!({ "upsert": true })),
                        },
                        Mount {
                            when: String::from("#!/jq\nany(.; .component[].mounts[]?.action | startswith(\"nginx::\"))"),
                            uri: Some(String::from("/api/py")),
                            src_uri: Some(String::from("#!/jq\n.component[] | select(.constraints | any([.kind, .required_variant] == [\"lang\", \"python\"])).dst_uri")),
                            action: String::from("mount::expose"),
                            action_args: None,
                        },
                        Mount {
                            when: String::from("#!/jq\nany(.; .component[].mounts[]?.action | startswith(\"nginx::\"))"),
                            uri: Some(String::from("/api/ruby")),
                            src_uri: Some(String::from("#!/jq\n.component[] | select(.constraints | any([.kind, .required_variant] == [\"lang\", \"ruby\"])).dst_uri")),
                            action: String::from("mount::expose"),
                            action_args: None,
                        },
                        Mount {
                            when: String::from("BUILD_TIME > 2024"),
                            uri: Some(String::from("/api/demo")),
                            src_uri: None, // 404
                            action: String::from("mount::expose"),
                            action_args: None,
                        },
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

    #[test]
    fn it_maybe_modify_string_via_shebang() {
        let vars: indexmap::IndexMap<String, String> = indexmap::indexmap! {
            String::from("config") => String::from("[\"Hello\", \"World\"]")
        };
        let untouched: &'static str = "untouched";
        let no_shebang = maybe_modify_string_via_shebang(&vars, untouched).unwrap();
        let jq_runs = maybe_modify_string_via_shebang(&vars, "#!/jq\n.[]").unwrap();
        assert_eq!(no_shebang, untouched);
        assert_eq!(jq_runs, String::from("\"Hello\"\n\"World\""));
    }
}
