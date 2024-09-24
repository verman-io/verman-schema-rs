use crate::commands::command::{Command, CommandKey};
use crate::commands::shared::merge_env;
use crate::errors::VermanSchemaError;
use crate::models::{CommonContent, Task};

#[derive(derive_more::Display)]
pub enum TaskKey {
    #[display("TASK_CURRENT_NAME")]
    CurrentName,

    #[display("TASK_PREVIOUS_NAME")]
    PreviousName,
}

pub async fn process_tasks_serially(
    pipeline_name: &String,
    tasks: &indexmap::IndexMap<String, Task>,
) -> Result<CommonContent, VermanSchemaError> {
    let mut shared_env_for_tasks = indexmap::IndexMap::<String, serde_json::Value>::new();

    let mut i = 0usize;
    for (task_name, task) in tasks.iter() {
        log::info!("Executing task {:#?}", task_name);
        shared_env_for_tasks.insert(
            TaskKey::CurrentName.to_string(),
            task_name.to_string().into(),
        );
        let common = Task {
            commands: task.commands.to_owned(),
            input_schema: task.input_schema.to_owned(),
            output_schema: task.output_schema.to_owned(),
            env: {
                merge_env(&mut shared_env_for_tasks, &task.env);
                Some(shared_env_for_tasks.clone())
            },
        }
        .process(pipeline_name, task_name, i)
        .await?;
        shared_env_for_tasks.swap_remove(TaskKey::CurrentName.to_string().as_str());
        shared_env_for_tasks.swap_remove(CommandKey::CurrentContent.to_string().as_str()); // `None` if key not found
                                                                                           // for security/sanity purposes, consider filtering env and only merge in necessary vars:
        if let Some(env) = common.env {
            shared_env_for_tasks.extend(env.to_owned());
        }
        shared_env_for_tasks.insert(
            TaskKey::PreviousName.to_string(),
            task_name.to_string().into(),
        );
        /*
        alternatively could add a `Vec<u8>` or `bytes` or `impl std::io::Read` field to
          the `CommonContent` struct
        */
        if let Some(value) = common.content.as_ref() {
            shared_env_for_tasks.insert(CommandKey::PreviousContent.to_string(), value.to_owned());
        }
        i += 1;
    }
    Ok(CommonContent {
        env: Some(shared_env_for_tasks.to_owned()),
        ..CommonContent::default()
    })
}

impl Task {
    async fn process(
        &self,
        pipeline_name: &String,
        task_name: &String,
        mut idx: usize,
    ) -> Result<CommonContent, VermanSchemaError> {
        let mut shared_env_for_cmds = match &self.env {
            Some(e) => e.to_owned(),
            None => indexmap::IndexMap::<String, serde_json::Value>::new(),
        };
        let mut last_result: Result<CommonContent, VermanSchemaError> =
            Err(VermanSchemaError::NotFound("`Command`s"));
        for command in &self.commands {
            last_result = Ok(command.process(&mut shared_env_for_cmds).await?);
            /* fail at first failing task, without retries or force continuing */

            // cache results
            Command::cache(
                pipeline_name,
                task_name,
                idx,
                &mut shared_env_for_cmds,
                &mut last_result,
            )?;
            idx += 1;
        }
        last_result
    }
}

#[cfg(test)]
#[path = "task_test.rs"]
mod task_test;
