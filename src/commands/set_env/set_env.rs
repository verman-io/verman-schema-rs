use crate::errors::VermanSchemaError;
use crate::models::CommonContent;

pub fn set_env(common_content: &CommonContent) -> Result<CommonContent, VermanSchemaError> {
    /* nothing needs to be done, envs are auto-merged onto next task */
    Ok(common_content.to_owned())
}

#[cfg(test)]
#[path = "set_env_test.rs"]
mod tests;
