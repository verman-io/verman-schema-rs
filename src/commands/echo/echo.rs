use crate::errors::VermanSchemaError;
use crate::models::CommonContent;
use serde_json::Value;
use std::ops::Deref;

pub fn echo(common_content: &CommonContent) -> Result<CommonContent, VermanSchemaError> {
    let content = common_content.content.to_owned();
    let env: indexmap::IndexMap<String, serde_json::Value> = match common_content.env {
        Some(ref _env) => _env.to_owned(),
        None => indexmap::IndexMap::<String, serde_json::Value>::new(),
    };
    let handle_success = |input_s: String| -> Result<CommonContent, VermanSchemaError> {
        if input_s.is_empty() {
            return Err(VermanSchemaError::NotFound("input to provide"));
        }
        let variables = {
            let mut hm = std::collections::HashMap::<String, String>::with_capacity(env.len());
            hm.extend(env.iter().filter_map(|(k, v)| match v {
                Value::String(s) => Some((k.to_owned(), s.to_owned())),
                _ => None,
            }));
            hm
        };

        let substituted = subst::substitute(input_s.as_str(), &variables)?;
        println!("{}", substituted);
        Ok(CommonContent {
            content: Some(serde_json::Value::String(String::from(substituted))),
            ..CommonContent::default()
        })
    };
    match content {
        None => match env.get("PREVIOUS_TASK_CONTENT") {
            Some(previous_task_output) => handle_success(match previous_task_output {
                either::Either::Left(s) => s.to_owned().into_bytes(),
                either::Either::Right(v) => v.to_owned(),
            }),
            None => Err(VermanSchemaError::NotFound("input to provide")),
        },
        Some(input) => {
            let input_s = {
                let mut input_s = input.to_owned().as_str().unwrap().to_string();
                if input_s.len() == 1 && input_s.get(1usize) == Some(String::from("-")) {
                    if let Some(previous_task_output) = env.get("PREVIOUS_TASK_CONTENT") {
                        input_s = match previous_task_output {
                            either::Either::Left(s) => s.to_owned().into_bytes(),
                            either::Either::Right(v) => v.to_owned(),
                        };
                    }
                }
                input_s
            };
            handle_success(input_s)
        }
    }
}

#[cfg(test)]
#[path = "echo_test.rs"]
mod tests;
