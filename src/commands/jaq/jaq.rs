use std::io::Write;

use crate::commands::command::CommandKey;
use crate::commands::shared::interpolate_input_else_get_prior_output;
use crate::errors::VermanSchemaError;
use crate::models::CommonContent;

mod jaq_utils;

pub fn jaq(common_content: &CommonContent) -> Result<CommonContent, VermanSchemaError> {
    let common_content_out = interpolate_input_else_get_prior_output(common_content, true)?;

    let content = common_content_out.content.clone();
    let env = common_content_out.env.clone();

    let input = Box::new(std::iter::once(Ok(match env {
        None => Err(VermanSchemaError::NotFound("Any content")),
        Some(envi) => {
            let json: serde_json::Value = envi
                .get(CommandKey::PreviousContent.to_string().as_str())
                .ok_or_else(|| VermanSchemaError::NotFound("Any content"))?
                .to_owned();
            Ok(jaq_json::Val::from(json))
        }
    }?)));
    let content_vec = content.ok_or_else(|| VermanSchemaError::NotFound("Any filter"))?;
    let filter = match content_vec {
        serde_json::Value::String(s) => Ok(s),
        _ => Err(VermanSchemaError::NotFound("String filter")),
    }?;

    let (vars, filter) = jaq_utils::vars_filter_from_code(filter.as_str())?;

    let mut buf = Vec::<u8>::new();
    let _result: bool = jaq_runner(&filter, vars.clone(), false, input, |v| {
        buf.write_all(v.to_string().as_bytes())
    })?
    .ok_or_else(|| VermanSchemaError::JaqStrError(String::from("`jaq_runner` failed")))?;
    /*assert!(_result);*/
    Ok(CommonContent {
        content: Some(match std::str::from_utf8(buf.as_slice()) {
            Ok(s) => serde_json::Value::String(s.to_string()),
            Err(_) => buf.into(),
        }),
        ..CommonContent::default()
    })
}

fn jaq_runner(
    filter: &jaq_core::Filter<jaq_core::Native<jaq_json::Val>>,
    vars: Vec<jaq_json::Val>,
    null_input: bool,
    iter: impl Iterator<Item = std::io::Result<jaq_json::Val>>,
    mut f: impl FnMut(jaq_json::Val) -> std::io::Result<()>,
) -> std::io::Result<Option<bool>> {
    let mut last = None;
    let iter = iter.map(|r| r.map_err(|e| e.to_string()));

    let iter = Box::new(iter) as Box<dyn Iterator<Item = _>>;
    let null = Box::new(core::iter::once(Ok(jaq_json::Val::Null))) as Box<dyn Iterator<Item = _>>;

    let iter = jaq_core::RcIter::new(iter);
    let null = jaq_core::RcIter::new(null);

    let ctx = jaq_core::Ctx::new(vars, &iter);

    for item in if null_input { &null } else { &iter } {
        let input = item.map_err(std::io::Error::other)?;
        for output in filter.run((ctx.clone(), input)) {
            use jaq_core::ValT;
            let output = output.map_err(|e| {
                std::io::Error::other(jaq_core::Error::<jaq_json::Val>::from(e).to_string())
            })?;
            last = Some(output.as_bool());
            f(output)?;
        }
    }
    Ok(last)
}

#[cfg(test)]
#[path = "jaq_test.rs"]
mod tests;
