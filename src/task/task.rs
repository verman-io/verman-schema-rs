use crate::commands::CommandArgs;
use crate::errors::VermanSchemaError;
use crate::models::{CommonContent, Task};

pub async fn process_tasks_serially(
    pipeline_name: &String,
    tasks: &indexmap::IndexMap<String, Task>,
) -> Result<CommonContent, VermanSchemaError> {
    let mut shared_env_for_tasks = indexmap::IndexMap::<String, serde_json::Value>::new();

    let mut i = 0usize;
    for (task_name, task) in tasks.iter() {
        log::info!("Executing task {:#?}", task_name);
        macro_rules! task_name_key {
            () => {
                "CURRENT_TASK_NAME"
            };
        }
        shared_env_for_tasks.insert(String::from(task_name_key!()), task_name.to_string().into());
        let common = Task::from_task_merge_env(task, &shared_env_for_tasks)
            .process(pipeline_name, task_name, i)
            .await?;
        shared_env_for_tasks.swap_remove(task_name_key!());
        macro_rules! previous_task_string_key {
            () => {
                "PREVIOUS_TASK_CONTENT"
            };
        }
        shared_env_for_tasks.swap_remove(previous_task_string_key!()); // `None` if key not found
                                                                       // for security/sanity purposes, consider filtering env and only merge in necessary vars:
        if let Some(env) = common.env {
            shared_env_for_tasks.extend(env.iter().map(|(k, v)| (k.to_owned(), v.to_owned())));
        }
        shared_env_for_tasks.insert(
            String::from("PREVIOUS_TASK_NAME"),
            task_name.to_string().into(),
        );
        /*
        alternatively could add a `Vec<u8>` or `bytes` or `impl std::io::Read` field to
          the `CommonContent` struct
        */
        if let Some(value) = common.content.as_ref() {
            shared_env_for_tasks
                .insert(String::from(previous_task_string_key!()), value.to_owned());
        }
        i += 1;
    }
    Ok(CommonContent {
        env: Some(shared_env_for_tasks),
        ..CommonContent::default()
    })
}

impl Task {
    async fn process(
        &self,
        pipeline_name: &String,
        task_name: &String,
        idx: usize
    ) -> Result<CommonContent, VermanSchemaError> {
        let mut shared_env_for_cmds = indexmap::IndexMap::<String, serde_json::Value>::new();
        let mut last_result: Result<CommonContent, VermanSchemaError> =
            Err(VermanSchemaError::NotFound("`Command`s"));
        for command in &self.commands {
            /* if !CommandName::VARIANTS.contains(command.cmd) {
                return Err(VermanSchemaError::NotInstalled(command.cmd.to_owned()))
            } */
            last_result = match command {
                CommandArgs::Echo(ref arg) => {
                    let common_with_merged_env =
                        Self::handle_shared_env_for_common_content_command(
                            &mut shared_env_for_cmds,
                            arg,
                        )?;
                    crate::commands::echo::echo(&common_with_merged_env)
                }
                CommandArgs::HttpClient(ref arg) => {
                    let mut common_with_merged_env = arg.to_owned();
                    common_with_merged_env.common_content.env =
                        match common_with_merged_env.common_content.env {
                            None => Some(shared_env_for_cmds.clone()),
                            Some(mut existing_env) => {
                                existing_env.extend(
                                    shared_env_for_cmds
                                        .iter()
                                        .map(|(k, v)| (k.to_owned(), v.to_owned())),
                                );
                                Some(existing_env)
                            }
                        };
                    let (_, common) =
                        crate::commands::http_client::http(&common_with_merged_env).await?;
                    Ok(common)
                }
                CommandArgs::SetEnv(ref arg) => {
                    let common_with_merged_env =
                        Self::handle_shared_env_for_common_content_command(
                            &mut shared_env_for_cmds,
                            arg,
                        )?;
                    crate::commands::set_env::set_env(&common_with_merged_env)
                }
                CommandArgs::Jaq(ref arg) => {
                    let mut common_with_merged_env = arg.to_owned();
                    common_with_merged_env.env = match common_with_merged_env.env {
                        None => Some(shared_env_for_cmds.clone()),
                        Some(mut existing_env) => {
                            existing_env.extend(
                                shared_env_for_cmds
                                    .iter()
                                    .map(|(k, v)| (k.to_owned(), v.to_owned())),
                            );
                            Some(existing_env)
                        }
                    };

                    Ok(crate::commands::jaq::jaq(&common_with_merged_env)?)
                }
                CommandArgs::Env(arg) => {
                    let common_with_merged_env =
                        Self::handle_shared_env_for_common_content_command(
                            &mut shared_env_for_cmds,
                            arg,
                        )?;
                    crate::commands::env::env(&common_with_merged_env)
                }
            }; /* fail at first failing task, without retries or force continuing */
            if let Ok(common) = last_result {
                if let Some(env_for_merge) = common.env.clone() {
                    shared_env_for_cmds.extend(env_for_merge.clone());
                }
                if let Some(vec_u8) = common.content.as_ref() {
                    let task_content = vec_u8.to_owned();

                    shared_env_for_cmds
                        .insert(String::from("PREVIOUS_TASK_CONTENT"), task_content.clone());
                    shared_env_for_cmds.insert(
                        String::from(format!("{}__{}[{}]_TASK_CONTENT", pipeline_name, task_name, idx)),
                        task_content.clone(),
                    );
                    shared_env_for_cmds.insert(
                        String::from(format!("{}__{}_TASK_CONTENT", pipeline_name, task_name)),
                        task_content,
                    );
                }
                last_result = Ok(common)
            }
        }
        last_result
    }

    fn handle_shared_env_for_common_content_command(
        shared_env_for_cmds: &mut indexmap::IndexMap<String, serde_json::Value>,
        arg: &CommonContent,
    ) -> Result<CommonContent, VermanSchemaError> {
        let mut common_with_merged_env = arg.to_owned();
        common_with_merged_env.env = match common_with_merged_env.env {
            None => Some(shared_env_for_cmds.clone()),
            Some(mut existing_env) => {
                existing_env.extend(
                    shared_env_for_cmds
                        .iter()
                        .map(|(k, v)| (k.to_owned(), v.to_owned())),
                );
                Some(existing_env)
            }
        };
        Ok(common_with_merged_env)
    }
}

#[cfg(test)]
#[path = "task_test.rs"]
mod task_test;
