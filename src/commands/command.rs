use crate::commands::shared::merge_env;
use crate::errors::VermanSchemaError;
use crate::models::{CommonContent, HttpCommandArgs};

#[derive(derive_more::Display)]
pub enum CommandKey {
    #[display("CMD_CURRENT_CONTENT")]
    CurrentContent,

    #[display("CMD_PREVIOUS_CONTENT")]
    PreviousContent,

    #[display("CMD_PREVIOUS_TYPE")]
    PreviousType,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "cmd")]
pub enum Command {
    Echo(CommonContent),
    Env(CommonContent),
    HttpClient(HttpCommandArgs),
    Interpolate(CommonContent),
    Jaq(CommonContent),
    SetEnv(CommonContent),
}

impl Default for Command {
    fn default() -> Self {
        Self::Echo(CommonContent::default())
    }
}

impl Command {
    pub async fn process(
        &self,
        mut shared_env_for_cmds: &mut indexmap::IndexMap<String, serde_json::Value>,
    ) -> Result<CommonContent, VermanSchemaError> {
        match self {
            Command::Echo(ref arg) => crate::commands::echo::echo(&CommonContent {
                env: {
                    merge_env(&mut shared_env_for_cmds, &arg.env);
                    Some(shared_env_for_cmds.clone())
                },
                content: arg.content.to_owned(),
            }),
            Command::Env(ref arg) => crate::commands::env::env(&CommonContent {
                env: {
                    merge_env(&mut shared_env_for_cmds, &arg.env);
                    Some(shared_env_for_cmds.clone())
                },
                content: arg.content.to_owned(),
            }),
            Command::HttpClient(ref arg) => crate::commands::http_client::http(&HttpCommandArgs {
                args: arg.args.to_owned(),
                common_content: CommonContent {
                    env: {
                        merge_env(&mut shared_env_for_cmds, &arg.common_content.env);
                        Some(shared_env_for_cmds.clone())
                    },
                    content: arg.common_content.content.to_owned(),
                },
                expectation: arg.expectation.to_owned(),
                deserialize_to: arg.deserialize_to.to_owned(),
            })
            .await
            .map(|(_, c)| c),
            Command::Interpolate(ref arg) => {
                crate::commands::interpolate::interpolate(&CommonContent {
                    env: {
                        merge_env(&mut shared_env_for_cmds, &arg.env);
                        Some(shared_env_for_cmds.clone())
                    },
                    content: arg.content.to_owned(),
                })
            }
            Command::Jaq(ref arg) => crate::commands::jaq::jaq(&CommonContent {
                env: {
                    merge_env(&mut shared_env_for_cmds, &arg.env);
                    Some(shared_env_for_cmds.clone())
                },
                content: arg.content.to_owned(),
            }),
            Command::SetEnv(ref arg) => crate::commands::set_env::set_env(&CommonContent {
                env: {
                    merge_env(&mut shared_env_for_cmds, &arg.env);
                    Some(shared_env_for_cmds.clone())
                },
                content: arg.content.to_owned(),
            }),
        }
    }

    pub fn cache(
        pipeline_name: &String,
        task_name: &String,
        idx: usize,
        shared_env_for_cmds: &mut indexmap::IndexMap<String, serde_json::Value>,
        last_result: &Result<CommonContent, VermanSchemaError>,
    ) -> Result<CommonContent, VermanSchemaError> {
        if let Ok(ref common) = last_result {
            Ok(CommonContent {
                env: match common.env {
                    Some(ref env_for_merge) => {
                        shared_env_for_cmds.extend(env_for_merge.clone());
                        Some(env_for_merge.to_owned())
                    }
                    None => None,
                },
                content: match common.content {
                    Some(ref content_val) => {
                        let val: serde_json::Value = match content_val {
                            serde_json::Value::String(s) => {
                                match serde_json::from_str(s.as_str()) {
                                    Ok(v) => v,
                                    Err(_) => serde_json::Value::String(s.to_owned()),
                                }
                            }
                            x @ _ => x.to_owned(),
                        };
                        shared_env_for_cmds
                            .insert(CommandKey::PreviousContent.to_string(), val.clone());
                        shared_env_for_cmds.insert(
                            String::from(format!(
                                "{}__{}[{}]_CMD_CONTENT",
                                pipeline_name, task_name, idx
                            )),
                            val.clone(),
                        );
                        shared_env_for_cmds.insert(
                            String::from(format!("{}__{}_CMD_CONTENT", pipeline_name, task_name)),
                            val.to_owned(),
                        );
                        Some(val.to_owned())
                    }
                    None => None,
                },
            })
        } else {
            Err(VermanSchemaError::NotFound("Nothing to cache"))
        }
    }
}
