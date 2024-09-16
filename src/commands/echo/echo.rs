use std::ops::Deref;

use crate::errors::VermanSchemaError;
use crate::models::CommonContent;

pub fn echo(common_content: &CommonContent) -> Result<CommonContent, VermanSchemaError> {
    let content = common_content.content.to_owned();
    let env: indexmap::IndexMap<String, either::Either<String, Vec<u8>>> = match common_content.env
    {
        Some(ref _env) => _env.to_owned(),
        None => indexmap::IndexMap::<String, either::Either<String, Vec<u8>>>::new(),
    };
    let handle_success = |input_vec: Vec<u8>| -> Result<CommonContent, VermanSchemaError> {
        if input_vec.is_empty() {
            return Err(VermanSchemaError::NotFound("input to provide"));
        }
        let variables = {
            let mut hm = std::collections::HashMap::<String, String>::with_capacity(env.len());
            hm.extend(
                env.iter()
                    .filter(|(_, v)| v.is_left())
                    .map(|(k, v)| (k.to_owned(), v.to_owned().left().unwrap())),
            );
            hm
        };

        let substituted = subst::substitute(std::str::from_utf8(input_vec.deref())?, &variables)?;
        println!("{}", substituted);
        Ok(CommonContent {
            content: Some(substituted.into_bytes()),
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
            let input_vec = {
                let mut input_v = input.to_owned();
                if input_v.len() == 1 && input_v.first() == Some("-".as_bytes().first().unwrap()) {
                    if let Some(previous_task_output) = env.get("PREVIOUS_TASK_CONTENT") {
                        input_v = match previous_task_output {
                            either::Either::Left(s) => s.to_owned().into_bytes(),
                            either::Either::Right(v) => v.to_owned(),
                        };
                    }
                }
                input_v
            };
            handle_success(input_vec)
        }
    }
}

#[cfg(test)]
#[path = "echo_test.rs"]
mod tests;
