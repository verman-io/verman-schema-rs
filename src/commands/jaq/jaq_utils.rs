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

pub(crate) fn vars_filter_from_code(
    code: &str,
) -> std::io::Result<(
    Vec<jaq_json::Val>,
    jaq_core::Filter<jaq_core::Native<jaq_json::Val>>,
)> {
    parse(
        &"<inline>",
        &code,
        &[String::from("ARGS"), String::from("ENV")],
        &[],
    )
    .map_err(|e| {
        let mut tmp = Vec::<u8>::with_capacity(e.len());
        e.iter().for_each(
            /* fold kept trying to make it a `&[u8]` */
            |elem| tmp.extend(format!("{:?}", elem).as_bytes()),
        );
        std::io::Error::other(std::str::from_utf8(tmp.as_slice()).unwrap())
    })
}

#[derive(Clone, Debug)]
enum Color {
    Yellow,
    Red,
}

type StringColors = Vec<(String, Option<Color>)>;

#[derive(Debug)]
struct Report {
    message: String,
    labels: Vec<(core::ops::Range<usize>, StringColors, Color)>,
}

type FileReports = (jaq_core::load::File<String>, Vec<Report>);

fn report_io(code: &str, (path, error): (&str, String)) -> Report {
    let path_range = jaq_core::load::span(code, path);
    Report {
        message: format!("could not load file {}: {}", path, error),
        labels: [(path_range, [(error, None)].into(), Color::Red)].into(),
    }
}

fn report_lex(code: &str, (expected, found): jaq_core::load::lex::Error<&str>) -> Report {
    // truncate found string to its first character
    let found = &found[..found.char_indices().nth(1).map_or(found.len(), |(i, _)| i)];

    let found_range = jaq_core::load::span(code, found);
    let found = match found {
        "" => [("unexpected end of input".to_string(), None)].into(),
        c => [("unexpected character ", None), (c, Some(Color::Red))]
            .map(|(s, c)| (s.into(), c))
            .into(),
    };
    let label = (found_range, found, Color::Red);

    let labels = match expected {
        jaq_core::load::lex::Expect::Delim(open) => {
            let text = [("unclosed delimiter ", None), (open, Some(Color::Yellow))]
                .map(|(s, c)| (s.into(), c));
            Vec::from([
                (jaq_core::load::span(code, open), text.into(), Color::Yellow),
                label,
            ])
        }
        _ => Vec::from([label]),
    };

    Report {
        message: format!("expected {}", expected.as_str()),
        labels,
    }
}

fn report_parse(code: &str, (expected, found): jaq_core::load::parse::Error<&str>) -> Report {
    let found_range = jaq_core::load::span(code, found);

    let found = if found.is_empty() {
        "unexpected end of input"
    } else {
        "unexpected token"
    };
    let found = [(found.to_string(), None)].into();

    Report {
        message: format!("expected {}", expected.as_str()),
        labels: Vec::from([(found_range, found, Color::Red)]),
    }
}

fn load_errors(errs: jaq_core::load::Errors<&str>) -> Vec<FileReports> {
    use jaq_core::load::Error;

    let errs = errs.into_iter().map(|(file, err)| {
        let code = file.code;
        let err = match err {
            Error::Io(errs) => errs.into_iter().map(|e| report_io(code, e)).collect(),
            Error::Lex(errs) => errs.into_iter().map(|e| report_lex(code, e)).collect(),
            Error::Parse(errs) => errs.into_iter().map(|e| report_parse(code, e)).collect(),
        };
        (file.map_code(|s| s.into()), err)
    });
    errs.collect()
}

fn report_compile(code: &str, (found, undefined): jaq_core::compile::Error<&str>) -> Report {
    let found_range = jaq_core::load::span(code, found);
    let message = format!("undefined {}", undefined.as_str());
    let found = [(message.clone(), None)].into();

    Report {
        message,
        labels: Vec::from([(found_range, found, Color::Red)]),
    }
}

fn compile_errors(errs: jaq_core::compile::Errors<&str>) -> Vec<FileReports> {
    let errs = errs.into_iter().map(|(file, errs)| {
        let code = file.code;
        let errs = errs.into_iter().map(|e| report_compile(code, e)).collect();
        (file.map_code(|s| s.into()), errs)
    });
    errs.collect()
}

fn invalid_data(e: impl std::error::Error + Send + Sync + 'static) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e)
}

fn json_slice(slice: &[u8]) -> impl Iterator<Item = std::io::Result<jaq_json::Val>> + '_ {
    let mut lexer = hifijson::SliceLexer::new(slice);
    core::iter::from_fn(move || {
        use hifijson::token::Lex;
        Some(jaq_json::Val::parse(lexer.ws_token()?, &mut lexer).map_err(invalid_data))
    })
}

/// Try to load file by memory mapping and fall back to regular loading if it fails.
fn load_file(
    path: impl AsRef<std::path::Path>,
) -> std::io::Result<Box<dyn core::ops::Deref<Target = [u8]>>> {
    let file = std::fs::File::open(path.as_ref())?;
    match unsafe { memmap2::Mmap::map(&file) } {
        Ok(mmap) => Ok(Box::new(mmap)),
        Err(_) => Ok(Box::new(std::fs::read(path)?)),
    }
}

fn json_array(path: impl AsRef<std::path::Path>) -> std::io::Result<jaq_json::Val> {
    json_slice(&load_file(path.as_ref())?).collect()
}

fn parse(
    path: &str,
    code: &str,
    vars: &[String],
    paths: &[std::path::PathBuf],
) -> Result<
    (
        Vec<jaq_json::Val>,
        jaq_core::Filter<jaq_core::Native<jaq_json::Val>>,
    ),
    Vec<FileReports>,
> {
    use jaq_core::compile::Compiler;
    use jaq_core::load::{import, Arena, File, Loader};

    let vars: Vec<_> = vars.iter().map(|v| format!("${v}")).collect();
    let arena = Arena::default();
    let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs())).with_std_read(paths);
    let path = path.into();
    let modules = loader
        .load(&arena, File { path, code })
        .map_err(load_errors)?;

    let mut vals = Vec::new();
    import(&modules, |p| {
        let path = p.find(paths, "json")?;
        vals.push(json_array(path).map_err(|e| e.to_string())?);
        Ok(())
    })
    .map_err(load_errors)?;

    let compiler = Compiler::default()
        .with_funs(jaq_std::funs().chain(jaq_json::funs()))
        .with_global_vars(vars.iter().map(|v| &**v));
    let filter = compiler.compile(modules).map_err(compile_errors)?;
    Ok((vals, filter))
}
