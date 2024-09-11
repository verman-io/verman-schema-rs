use crate::constants::VARS;
use crate::execute_shebang;
use crate::models::{
    Component, Constraint, Mount, ProtocolConfiguration, Root, ServerConfiguration, State,
    StateValues, VermanConfig,
};

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
                        src_uri: Some(String(".component[] | select(.constraints | any([.kind, .required_variant] == [\"lang\", \"ruby\"])).dst_uri")),
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
