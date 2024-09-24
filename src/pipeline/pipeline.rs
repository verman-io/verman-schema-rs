use crate::commands::shared::merge_env;
use crate::errors::VermanSchemaError;
use crate::models::{CommonContent, Pipeline, Task};
use crate::task::task::process_tasks_serially;

impl Default for Pipeline {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: String::new(),
            description: String::new(),
            url: String::new(),
            engine_version: String::from("0.1.0"),
            env: None,
            pipe: None,
            tasks: None,
            schemas: None,
        }
    }
}

impl Pipeline {
    pub async fn process(&self) -> Result<CommonContent, VermanSchemaError> {
        let pretty_name = format!(
            "{}@{} from {}\n{}",
            self.name, self.version, self.url, self.description
        );
        log::info!("Started processing {}", pretty_name);
        let common = match &self.tasks {
            Some(tasks) => {
                let tasks_with_merged_env = tasks
                    .iter()
                    .map(|(name, task)| {
                        (
                            name.to_owned(),
                            Task {
                                commands: task.commands.to_owned(),
                                input_schema: task.input_schema.to_owned(),
                                output_schema: task.output_schema.to_owned(),
                                env: {
                                    let mut task_env = task
                                        .env
                                        .to_owned()
                                        .unwrap_or_else(|| indexmap::IndexMap::new());
                                    merge_env(&mut task_env, &self.env);
                                    Some(task_env)
                                },
                            },
                        )
                    })
                    .collect();
                process_tasks_serially(&self.name, &tasks_with_merged_env).await?
            }
            None => {
                log::warn!("No tasks found in pipeline");
                CommonContent::default()
            }
        };
        log::info!("Finished processing {}", pretty_name);
        Ok(common)
    }
}

#[cfg(test)]
#[path = "pipeline_test.rs"]
mod tests;
