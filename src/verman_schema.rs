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
