#![feature(iter_collect_into)]

mod constants;

mod error;
mod utils;

mod models;


extern crate jaq_core;
extern crate serde;
#[macro_use]
extern crate lazy_static;

use crate::error::VermanSchemaError;

fn jq<'a>(
    vars: &'a mut indexmap::IndexMap<String, String>,
    filter: &str,
) -> Result<String, VermanSchemaError> {
    jaq_utils::give(serde_json::Value::Object(vars), filter)
}

pub fn execute_shebang<'a>(
    vars: &'a mut indexmap::IndexMap<String, String>,
    filter: &'a str,
) -> Result<(), VermanSchemaError> {
    print!("filter: \"{}\"\n", filter);
    print!("vars: {:#?}\n", vars);

    let process = |first_line: &str, rest: &str| -> Result<(), VermanSchemaError> {
        let shebang: String = if !first_line.starts_with("#!/") && vars.contains_key("SHELL") {
            vars["SHELL"].to_owned()
        } else {
            first_line.into_string()
        };
        match shebang.as_str() {
            "#!/jq" => {
                vars.insert(String::from("THIS_NO_SHEBANG"), String::from(rest));
                vars.insert(String::from("SHELL"), String::from(shebang));
                match jq(vars, filter) {
                    Ok(jq_ified) => {
                        vars["THIS"] = jq_ified;
                        Ok(())
                    }
                    Err(e) => Err(e),
                    // `^ the `?` operator cannot be applied to type `Cow<'_, _>``
                }
            }
            "#!/echo" => {
                vars.insert(String::from("SHELL"), String::from(shebang));
                Ok(())
            }
            _ => unimplemented!("TODO: Generic shebang handling for: {}", first_line),
        }
    };

    let get_rest_key = || -> &'static str {
        if vars.contains_key("THIS_NO_SHEBANG") {
            "THIS_NO_SHEBANG"
        } else {
            "THIS"
        }
    };

    if let Some(first_nl) = vars["THIS"].find('\n') {
        if !vars.contains_key("THIS_FIRST_LINE") {
            vars["THIS_FIRST_LINE"] = String::from(&vars["THIS"][..first_nl]);
        }
        process(&vars["THIS_FIRST_LINE"], &vars["THIS"][first_nl + 1..])
    } else if vars.contains_key("THIS_FIRST_LINE") {
        process(&vars["THIS_FIRST_LINE"], &vars[get_rest_key()])
    } else if vars.contains_key("SHELL") {
        process(&vars["SHELL"], &vars[get_rest_key()])
    } else {
        Ok(())
    }
}

fn get_shell<'a>(
    vars: &'a indexmap::IndexMap<String, String>,
) -> Result<&'a str, VermanSchemaError> {
    if vars.contains_key("SHELL") {
        Ok(&vars["SHELL"])
    } else if vars.contains_key("THIS_FIRST_LINE") {
        Ok(&vars["THIS_FIRST_LINE"])
    } else if let Some(first_nl) = vars["THIS"].find('\n') {
        Ok(&vars["THIS"][..first_nl])
    } else {
        Err(VermanSchemaError::UnexpectedEmptiness)
    }
}

pub fn prepend_vars(
    mut input: String,
    vars: indexmap::IndexMap<String, String>,
) -> Result<(), VermanSchemaError> {
    const VARS_TO_IGNORE: &'static [&'static str] = &["THIS", "THIS_FIRST_LINE", "SHELL"];
    let shell: &str = get_shell(&vars)?;

    match shell {
        "#!/jq" => {
            let defs: String = utils::join(
                "\n",
                vars.iter()
                    .filter(|(k, _)| VARS_TO_IGNORE.contains(k.as_ref()))
                    .map(|(k, v)| {
                        utils::Concat((
                            k,
                            "=",
                            v.trim().parse::<f64>().map_or_else(
                                |_err| utils::Concat(('"', v, '"')).to_string(),
                                |_ok| *v,
                            ),
                            ";",
                        ))
                    }),
            );
            input.insert_str(0, defs.as_str());
            Ok(())
        }
        "#!/echo" => Ok(()),
        _ => Err(VermanSchemaError::UnexpectedEmptiness),
    }?;
    Ok(())
}

#[cfg(test)]
#[path = "test_verman_schema.rs"]
mod tests;
mod jaq_utils;
