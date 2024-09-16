use crate::errors::VermanSchemaError;
use crate::models::{CommonContent, Pipeline};
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
            pipe: vec![],
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
            Some(ref tasks) => process_tasks_serially(&self.name, tasks).await?,
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
