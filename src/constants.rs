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
