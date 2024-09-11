#![feature(iter_collect_into)]

mod utils;

extern crate jaq_core;
extern crate jaq_interpret;
extern crate serde;
#[macro_use]
extern crate lazy_static;

/// These 4 constants + `THIS` referring holding contents of current config file
/// are made accessible to each script

pub const ARCH: &'static str = std::env::consts::ARCH;
pub const FAMILY: &'static str = std::env::consts::FAMILY;
pub const OS: &'static str = std::env::consts::OS;

lazy_static! {
    pub static ref BUILD_TIME: std::time::SystemTime = std::time::SystemTime::now();
    // RFC3339 format
    pub static ref BUILD_TIME_STR: String = {
        let since_epoch: std::time::Duration = BUILD_TIME.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards");
        let seconds: u64 = since_epoch.as_secs();
        let nanos: u32 = since_epoch.subsec_nanos();
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}Z",
            1970 + seconds / 31536000,
            (seconds % 31536000) / 2592000,
            (seconds % 2592000) / 86400,
            (seconds % 86400) / 3600,
            (seconds % 3600) / 60,
            seconds % 60,
            nanos
        )
    };

    pub static ref VARS: indexmap::IndexMap<String, String> = indexmap::indexmap! {
            String::from("ARCH") => String::from(ARCH),
            String::from("FAMILY") => String::from(FAMILY),
            String::from("OS") => String::from(OS),
            String::from("BUILD_TIME") => String::from(BUILD_TIME_STR.as_str()),
            String::from("THIS") => String::from("#!/jq\n[\"Hello\", \"World\"]")
    };
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
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

#[derive(Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct VermanConfig {
    /// If shebang is provided, takes priority; otherwise this default is used
    /// Special lines:
    /// - `"#!/echo` simply outputs the lines after shebang
    /// - `"#!/jq"` uses internal `jq` implementation (`jaq`) if feature `jaq` else errs
    /// - `"#!/deno"` uses internal deno dependency (WASM, js, ts) if feature `js` enabled else errs
    /// - `"#!/wasm"` uses internal deno dependency (WASM, js, ts) if feature `js` enabled else errs
    /// - `"#!/js"` uses internal deno dependency (WASM, js, ts) if feature `js` enabled else errs
    /// - `"#!/lua"` uses internal Lua dependency if feature `lua` enabled else errs
    /// - `"#!/python"` uses internal Python dependency if feature `python` enabled else errs
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

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct StateValues {
    pub kind: Option<String>,
    pub install: Option<State>,
    pub remove: Option<State>,
    pub start: Option<State>,
    pub stop: Option<State>,
}

#[derive(Debug, Default, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
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

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ServerConfiguration {
    pub kind: String,
    pub versions: Option<Vec<String>>,
    pub server_priority: Option<Vec<String>>,
    /// environment variables. Priority: `ServerConfiguration` | `component`; `Root`; system.
    pub env_vars: Option<indexmap::IndexMap<String, String>>,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
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
#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Component {
    pub src_uri: Option<String>,
    pub dst_uri: Option<String>,
    pub constraints: Vec<Constraint>,
    /// environment variables. Priority: `ServerConfiguration` | `Component`; `Root`; system.
    pub env_vars: Option<indexmap::IndexMap<String, String>>,
    pub mounts: Option<Vec<Mount>>,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
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

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Constraint {
    pub kind: String,
    pub required_variant: Option<String>,
    pub required_version: Option<String>,
}

/// OSs from https://github.com/rust-lang/rust/blob/1.77.0/library/std/src/env.rs#L947-L961
#[derive(
    Debug, Default, Clone, PartialEq, Eq, Hash, serde_derive::Serialize, serde_derive::Deserialize,
)]
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

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct VendorVersion {
    pub vendor: Option<String>,
    pub version: Option<String>,
}

/*
pub fn eval_field(s: Box<dyn Into<String>>) -> String {
    s
}
*/

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Jaq(jaq_interpret::Error),
    IOError(std::io::Error),
    /// Depressing
    UnexpectedEmptiness,
    ParsingError,
    CompilationError,
}

impl From<jaq_interpret::Error> for Error {
    fn from(err: jaq_interpret::Error) -> Self {
        Error::Jaq(err)
    }
}

fn jq<'a>(vars: &'a mut indexmap::IndexMap<String, String>, filter: &str) -> Result<String, Error> {
    let mut defs = jaq_interpret::ParseCtx::new(Vec::new());

    let (f, errs) = jaq_parse::parse(filter, jaq_parse::main());
    if !errs.is_empty() {
        return Err(Error::ParsingError);
    }

    let f = defs.compile(f.unwrap());
    if !defs.errs.is_empty() {
        return Err(Error::CompilationError);
    }

    let inputs = jaq_interpret::RcIter::new(core::iter::empty());

    let val = <jaq_interpret::Val as From<serde_json::Value>>::from(serde_json::json!(
        vars[if vars.contains_key("THIS_NO_SHEBANG") {
            "THIS_NO_SHEBANG"
        } else {
            "THIS"
        }]
    ));

    let mut out = jaq_interpret::FilterT::run(
        &f,
        (
            jaq_interpret::Ctx::new([], &inputs),
            val,
            // jaq_interpret::Val::from(vars["config"].clone()),
        ),
    );

    let mut results = Vec::<String>::new();
    let mut errors = Vec::<jaq_interpret::Error>::new();
    let mut last_err: Option<jaq_interpret::Error> = None;
    while let Some(val) = out.next() {
        match val {
            Ok(jaq_interpret::Val::Str(s)) => {
                println!("one row: \"{}\"\n", &s);
                results.push(s.to_string())
            }
            Ok(jaq_interpret::Val::Null) => todo!(),
            Ok(jaq_interpret::Val::Bool(_)) => todo!(),
            Ok(jaq_interpret::Val::Int(_)) => todo!(),
            Ok(jaq_interpret::Val::Float(_)) => todo!(),
            Ok(jaq_interpret::Val::Num(_)) => todo!(),
            Ok(jaq_interpret::Val::Arr(_)) => todo!(),
            Ok(jaq_interpret::Val::Obj(_)) => todo!(),
            Err(e) => {
                last_err = Some(e.clone());
                errors.push(e)
            }
        }
    }
    println!("results: {:?}", results);
    if let Some(first_error) = errors.first() {
        if errors.len() == 1 {
            Err(Error::Jaq(last_err.unwrap()))
        } else {
            /* temporary hack */
            Err(Error::IOError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{}", first_error),
            )))
        }
    } else {
        Ok(String::from_iter(results.into_iter()))
    }
}

pub fn execute_shebang<'a>(
    vars: &'a mut indexmap::IndexMap<String, String>,
    filter: &'a str,
) -> Result<(), Error> {
    print!("filter: \"{}\"\n", filter);
    print!("vars: {:#?}\n", vars);

    let process =
        |first_line: &str, rest: &str| -> Result<(), Error> {
            let shebang: String = if !first_line.starts_with("#!/") && vars.contains_key("SHELL") {
                vars["SHELL"]
            } else {
                first_line.into_string()
            };
            match shebang {
                "#!/jq" => {
                    vars.insert(String::from("THIS_NO_SHEBANG"), String::from(rest));
                    vars.insert(String::from("SHELL"), String::from(shebang));
                    match jq(vars, filter) {
                        Ok(jq_ified) => { vars["THIS"] = jq_ified; Ok(()) },
                        Err(e) => Err(e),
                        // `^ the `?` operator cannot be applied to type `Cow<'_, _>``
                    }
                }
                "#!/echo" => { vars.insert(String::from("SHELL"), String::from(shebang)); Ok(()) },
                _ => unimplemented!("TODO: Generic shebang handling for: {}", first_line),
            }
        };

    let get_rest_key = || -> &'static str {
        if vars.contains_key("THIS_NO_SHEBANG") {
            "THIS_NO_SHEBANG"
        } else {
            "THIS"
        }
    };

    if let Some(first_nl) = vars["THIS"].find('\n') {
        if !vars.contains_key("THIS_FIRST_LINE") {
            vars["THIS_FIRST_LINE"] = String::from(&vars["THIS"][..first_nl]);
        }
        process(&vars["THIS_FIRST_LINE"], &vars["THIS"][first_nl + 1..])
    } else if vars.contains_key("THIS_FIRST_LINE") {
        process(&vars["THIS_FIRST_LINE"], &vars[get_rest_key()])
    } else if vars.contains_key("SHELL") {
        process(&vars["SHELL"], &vars[get_rest_key()])
    } else {
        Ok(())
    }
}

fn get_shell<'a>(vars: &'a indexmap::IndexMap<String, String>) -> Result<&'a str, Error> {
    if vars.contains_key("SHELL") {
        Ok(&vars["SHELL"])
    } else if vars.contains_key("THIS_FIRST_LINE") {
        Ok(&vars["THIS_FIRST_LINE"])
    } else if let Some(first_nl) = vars["THIS"].find('\n') {
        Ok(&vars["THIS"][..first_nl])
    } else {
        Err(Error::UnexpectedEmptiness)
    }
}

pub fn prepend_vars(
    mut input: String,
    vars: indexmap::IndexMap<String, String>,
) -> Result<(), Error> {
    const VARS_TO_IGNORE: &'static [&'static str] = &["THIS", "THIS_FIRST_LINE", "SHELL"];
    let shell: &str = get_shell(&vars)?;

    match shell {
        "#!/jq" => {
            let defs: String = utils::join(
                "\n",
                vars.iter()
                    .filter(|(k, _)| VARS_TO_IGNORE.contains(k.as_ref()))
                    .map(|(k, v)| {
                        utils::Concat((
                            k,
                            "=",
                            v.trim().parse::<f64>().map_or_else(
                                |_err| utils::Concat(('"', v, '"')).to_string(),
                                |_ok| *v,
                            ),
                            ";",
                        ))
                    }),
            );
            input.insert_str(0, defs.as_str());
            Ok(())
        }
        "#!/echo" => Ok(()),
        _ => Err(Error::UnexpectedEmptiness),
    }?;
    Ok(())
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
                    dst_uri: Some(String::from("if $OS == \"windows\" then \"\\\\.\\pipe\\PipeName\" else \"unix:///var/run/my-socket.sock\"")),
                    constraints: vec![
                        Constraint {
                            kind: String::from("lang"),
                            required_variant: Some(String::from("ruby")),
                            required_version: Some(String::from(">3.1.2, <3.2")),
                        },
                        Constraint {
                            kind: String::from("OS"),
                            required_variant: Some(String::from("$OS | in({\"linux\" || \"windows\"})")),
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
                            when: String::from("$OS == \"windows\""),
                            uri: Some(String::from("file://win_nginx.conf")),
                            src_uri: None,
                            action: String::from("nginx::make_site_available"),
                            action_args: Some(serde_json::json!({ "upsert": true })),
                        },
                        Mount {
                            when: String::from("any(.; .component[].mounts[]?.action | startswith(\"nginx::\"))"),
                            uri: Some(String::from("/api/py")),
                            src_uri: Some(String::from(".component[] | select(.constraints | any([.kind, .required_variant] == [\"lang\", \"python\"])).dst_uri")),
                            action: String::from("mount::expose"),
                            action_args: None,
                        },
                        Mount {
                            when: String::from("any(.; .component[].mounts[]?.action | startswith(\"nginx::\"))"),
                            uri: Some(String::from("/api/ruby")),
                            src_uri: Some(String::from(".component[] | select(.constraints | any([.kind, .required_variant] == [\"lang\", \"ruby\"])).dst_uri")),
                            action: String::from("mount::expose"),
                            action_args: None,
                        },
                        Mount {
                            when: String::from("$BUILD_TIME > 2024"),
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
        let mut vars: indexmap::IndexMap<String, String> = VARS.clone();
        const UNTOUCHED: &'static str = "UNTOUCHED";
        let mut vars_unmodified: indexmap::IndexMap<String, String> = VARS.clone();
        vars_unmodified["THIS"] = String::from(UNTOUCHED);
        let no_shebang: std::borrow::Cow<str> =
            execute_shebang(&mut vars_unmodified, UNTOUCHED).unwrap();
        let jq_runs: std::borrow::Cow<str> = execute_shebang(&mut vars, ".[0]").unwrap();
        assert_eq!(no_shebang, UNTOUCHED);
        assert_eq!(jq_runs, String::from("\"Hello\"\n\"World\""));
    }
}
