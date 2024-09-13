use std::io::Write;

use crate::commands::jaq::jaq_utils::vars_filter_from_code;
use crate::error::VermanSchemaError;

mod jaq_utils;

pub fn jaq<'a>(content: Vec<jaq_json::Val>, query: &str) -> Result<String, VermanSchemaError> {
    let inputs = Box::new(std::iter::once(Ok(jaq_json::Val::Arr(std::rc::Rc::new(
        content,
    )))));

    let (vars, filter) = vars_filter_from_code(query).unwrap();

    let mut buf = Vec::<u8>::new();
    let _result: bool = jaq_runner(&filter, vars.clone(), false, inputs, |v| {
        buf.write_all(v.to_string().as_bytes())
    })?
    .ok_or_else(|| VermanSchemaError::NotFound(""))?;
    Ok(String::from(std::str::from_utf8(buf.as_slice())?))
    /*assert!(_result);*/
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
        let input = item.map_err(|e| std::io::Error::other(e))?;
        //println!("Got {:?}", input);
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
