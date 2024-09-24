use crate::commands::shared::interpolate_input_else_get_prior_output;
use crate::errors::VermanSchemaError;
use crate::models::CommonContent;

pub fn env(common_content: &CommonContent) -> Result<CommonContent, VermanSchemaError> {
    let common_content_out = interpolate_input_else_get_prior_output(common_content)?;
    if let Some(ref env) = common_content_out.env {
        env.iter()
            .for_each(|(k, v)| println!("{}={}", k, serde_json::to_string(v).unwrap()))
    }
    Ok(common_content_out)
}

#[cfg(test)]
#[path = "env_test.rs"]
mod tests;
