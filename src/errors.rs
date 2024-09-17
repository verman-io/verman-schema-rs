use std::string::ToString;

#[derive(derive_more::Display, derive_more::Error, derive_more::From, Debug)]
#[repr(u16)]
pub enum VermanSchemaError {
    #[error(ignore)]
    #[from(skip)]
    #[display("NotFound({_0:#?})")]
    NotFound(&'static str) = 404,

    #[error(ignore)]
    #[from(skip)]
    #[display("{_0:?}")]
    Task(String) = 594,

    #[error(ignore)]
    #[from(skip)]
    #[display("{_0:?}")]
    TaskFailedToStart(String) = 597,

    #[error(ignore)]
    #[display("{_0:?}")]
    HttpError(u16) = 598,

    #[error(ignore)]
    #[from(skip)]
    #[display("{_0:?}")]
    NotInstalled(String) = 599,

    // ************************
    // * Library level errors *
    // ************************
    #[display("`std::io::Error` error. {error:?}")]
    StdIoError { error: std::io::Error } = 700,

    #[error(ignore)]
    #[display("{_0:?}")]
    ExitCode(std::process::ExitCode) = 710,

    #[display("`toml::de::Error` error. {error:?}")]
    TomlDeError { error: toml::de::Error } = 720,

    #[display("`serde_json::Error` error. {error:?}")]
    SerdeJsonError { error: serde_json::Error } = 721,

    #[display("`reqwest::Error` error. {error:?}")]
    ReqwestError { error: reqwest::Error } = 732,

    #[display("`http::header::InvalidHeaderName` error. {error:?}")]
    InvalidHeaderName {
        error: http::header::InvalidHeaderName,
    } = 733,

    #[display("`http::header::InvalidHeaderValue` error. {error:?}")]
    InvalidHeaderValue {
        error: http::header::InvalidHeaderValue,
    } = 734,

    #[display("`http::method::InvalidMethod` error. {error:?}")]
    InvalidMethod { error: http::method::InvalidMethod } = 735,

    #[display("`http::uri::InvalidUri` error. {error:?}")]
    InvalidUri { error: http::uri::InvalidUri } = 736,

    #[display("`serde_json_extensions::error::Error` error. {error:?}")]
    SerdeJsonExtensionsError {
        error: serde_json_extensions::error::Error,
    } = 737,

    #[display("`subst::Error` error. {error:?}")]
    SubstError { error: subst::Error } = 738,

    #[display("`std::str::Utf8Error` error. {error:?}")]
    Utf8Error { error: std::str::Utf8Error } = 739,

    #[display("`jaq_core::compile::Errors` error. {error:?}")]
    JaqCoreError {
        error: jaq_core::compile::Errors<String>,
    } = 740,

    #[display("`jaq_json::Error` error. {error:?}")]
    JaqJsonError { error: jaq_json::Error } = 741,

    #[error(ignore)]
    #[from(skip)]
    #[display("`jaq` str error. {_0}")]
    JaqStrError(String) = 742,
}

impl VermanSchemaError {
    fn discriminant(&self) -> u16 {
        unsafe { *<*const _>::from(self).cast::<u16>() }
    }
}

impl std::process::Termination for VermanSchemaError {
    fn report(self) -> std::process::ExitCode {
        if let VermanSchemaError::ExitCode(exit_code) = self {
            return exit_code;
        }
        let status_code = self.discriminant();
        if status_code > u8::MAX as u16 {
            eprintln!("exit code {}", status_code);
            std::process::ExitCode::FAILURE
        } else {
            std::process::ExitCode::from(status_code as u8)
        }
    }
}

pub enum SuccessOrVermanSchemaError<T> {
    Ok(T),
    Err(VermanSchemaError),
}

impl<T> From<Result<T, VermanSchemaError>> for SuccessOrVermanSchemaError<T> {
    fn from(value: Result<T, VermanSchemaError>) -> Self {
        match value {
            Ok(val) => SuccessOrVermanSchemaError::Ok(val),
            Err(error) => SuccessOrVermanSchemaError::Err(error),
        }
    }
}

// Can't use `Result` because
// [E0117] Only traits defined in the current crate can be implemented for arbitrary types
impl<T: std::any::Any> std::process::Termination for SuccessOrVermanSchemaError<T> {
    fn report(self) -> std::process::ExitCode {
        const PROCESS_EXIT_CODE: fn(i32) -> std::process::ExitCode = |e: i32| {
            if e > u8::MAX as i32 {
                eprintln!("exit code {}", e);
                std::process::ExitCode::FAILURE
            } else {
                std::process::ExitCode::from(e as u8)
            }
        };

        /* const REPORT: fn(impl Termination + ToString + Sized) -> ExitCode = |err: impl std::process::Termination + std::string::ToString| -> std::process::ExitCode {
            eprintln!("{}", err.to_string());
            err.report()
        }; */

        match self {
            SuccessOrVermanSchemaError::Ok(e)
                if std::any::TypeId::of::<T>()
                    == std::any::TypeId::of::<std::process::ExitCode>() =>
            {
                *(&e as &dyn std::any::Any)
                    .downcast_ref::<std::process::ExitCode>()
                    .unwrap()
            }
            SuccessOrVermanSchemaError::Ok(_) => std::process::ExitCode::SUCCESS,
            SuccessOrVermanSchemaError::Err(err) => match err {
                VermanSchemaError::StdIoError { ref error } if error.raw_os_error().is_some() => {
                    let e = unsafe { error.raw_os_error().unwrap_unchecked() };
                    eprintln!("{}", e.to_string());
                    PROCESS_EXIT_CODE(e)
                }
                VermanSchemaError::ExitCode(exit_code) => exit_code,
                _ => {
                    eprintln!("{}", err.to_string());
                    err.report()
                }
            },
        }
    }
}

// TODO: Get `Into<VermanSchemaError>` syntax working
impl std::ops::FromResidual<Result<std::convert::Infallible, VermanSchemaError>>
    for SuccessOrVermanSchemaError<std::process::ExitCode>
{
    fn from_residual(residual: Result<std::convert::Infallible, VermanSchemaError>) -> Self {
        SuccessOrVermanSchemaError::Err(residual./*into_*/err().unwrap())
    }
}

impl std::ops::FromResidual<Result<std::convert::Infallible, std::io::Error>>
    for SuccessOrVermanSchemaError<std::process::ExitCode>
{
    fn from_residual(residual: Result<std::convert::Infallible, std::io::Error>) -> Self {
        SuccessOrVermanSchemaError::Err(VermanSchemaError::from(residual./*into_*/err().unwrap()))
    }
}
