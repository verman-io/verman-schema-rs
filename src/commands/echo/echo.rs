use crate::commands::shared::interpolate_input_with_env;
use crate::errors::VermanSchemaError;
use crate::models::CommonContent;

pub fn echo(common_content: &CommonContent) -> Result<CommonContent, VermanSchemaError> {
    let common_content_out = interpolate_input_with_env(common_content)?;
    if let Some(ref content) = common_content_out.content {
        match content {
            serde_json::Value::String(s) => println!("{}", s),
            x @ _ => println!("{}", serde_json::to_string(x).unwrap()),
        }
    }
    Ok(common_content_out)
}

#[cfg(test)]
#[path = "echo_test.rs"]
mod tests;
