use crate::error::VermanSchemaError;

fn jaq_error_mapper(error: jaq_core::compile::Errors<&str>) -> VermanSchemaError {
    VermanSchemaError::JaqStrError(
        error
            .iter()
            .map(|(f, err_vec)| -> String {
                String::from(format!(
                    "{:?}{:?}",
                    f,
                    err_vec
                        .iter()
                        .map(|er| format!("{:?}", er))
                        .reduce(|s0, s1| s0 + s1.as_str())
                        .unwrap_or(String::new())
                ))
            })
            .reduce(|s0, s1| s0 + s1.as_str())
            .unwrap_or(String::new()),
    )
}

// Next 4 definitions are from https://github.com/01mf02/jaq/blob/7b9ce5b/jaq-core/tests/common/mod.rs
// but modified to not panic
fn yields(
    x: jaq_json::Val,
    code: &str,
    ys: impl Iterator<Item = jaq_json::ValR>,
) -> Result<(), VermanSchemaError> {
    use jaq_core::load::{Arena, File, Loader};
    use jaq_core::{Compiler, Native};

    let arena = Arena::default();
    let loader = Loader::new([]);
    let path = "".into();
    let modules = loader
        .load(&arena, File { path, code })
        .map_err(|e| -> VermanSchemaError { e.iter().map() });
    let filter = Compiler::<_, Native<_>>::default()
        .compile(modules)
        .map_err(jaq_error_mapper)?;
    Ok(filter.yields(x, ys))
}

pub fn fail(x: serde_json::Value, f: &str, err: jaq_json::Error) -> Result<(), VermanSchemaError> {
    yields(x.into(), f, core::iter::once(Err(err)))
}

pub fn give(x: serde_json::Value, f: &str, y: serde_json::Value) -> Result<(), VermanSchemaError> {
    yields(x.into(), f, core::iter::once(Ok(y.into())))
}

pub fn gives<const N: usize>(
    x: serde_json::Value,
    f: &str,
    ys: [serde_json::Value; N],
) -> Result<(), VermanSchemaError> {
    yields(x.into(), f, ys.into_iter().map(|y| Ok(y.into())))
}
