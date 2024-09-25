use crate::commands::shared::{interpolate_input_else_get_prior_output, make_subst_map};
use crate::errors::VermanSchemaError;
use crate::models::CommonContent;

pub fn interpolate(common_content: &CommonContent) -> Result<CommonContent, VermanSchemaError> {
    let common_content_out = interpolate_input_else_get_prior_output(common_content, true)?;
    if let Some(ref content) = common_content_out.content {
        if let Some(ref env) = common_content.env {
            let variables = make_subst_map(env);
            let (mut content_s, was_str) = match content {
                serde_json::Value::String(s) => (s.to_owned(), true),
                val @ _ => (serde_json::to_string(val)?, false),
            };
            for _ in 0..10 {
                content_s = subst::substitute(&content_s, &variables)?;
            }
            Ok(CommonContent {
                content: Some(if was_str {
                    serde_json::Value::String(content_s)
                } else {
                    serde_json::from_str(content_s.as_str())?
                }),
                env: Some(env.to_owned()),
            })
        } else {
            Ok(common_content.to_owned())
        }
    } else {
        Ok(common_content.to_owned())
    }
}

#[cfg(test)]
#[path = "interpolate_test.rs"]
mod tests;
