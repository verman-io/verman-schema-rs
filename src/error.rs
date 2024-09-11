#[derive(
    strum::AsRefStr, strum::Display, strum::EnumDiscriminants, strum::IntoStaticStr, Debug,
)]
#[repr(u16)]
#[non_exhaustive]
pub enum VermanSchemaError {
    #[strum(to_string = "NotFound({0:#?})")]
    NotFound(&'static str) = 404,

    /// Depressing
    #[strum(to_string = "UnexpectedEmptiness")]
    UnexpectedEmptiness=680,
    #[strum(to_string = "ParsingError")]
    ParsingError=681,
    #[strum(to_string = "CompilationError")]
    CompilationError=682,

    // ************************
    // * Library level errors *
    // ************************
    #[strum(to_string = "`std::io::Error` error. {error:?}")]
    StdIoError { error: std::io::Error } = 700,

    #[strum(to_string = "{0:?}")]
    ExitCode(std::process::ExitCode) = 710,

    #[strum(to_string = "`std::str::Utf8Error` error. {error:?}")]
    Utf8Error { error: std::str::Utf8Error } = 739,

    #[strum(to_string = "`jaq_core::compile::Errors` error. {error:?}")]
    JaqCoreError {
        error: jaq_core::compile::Errors<String>,
    } = 740,

    #[strum(to_string = "`jaq_json::Error` error. {error:?}")]
    JaqJsonError { error: jaq_json::Error } = 741,

    #[strum(to_string = "`jaq` str error. {0}")]
    JaqStrError(String) = 742,
}

impl VermanSchemaError {
    fn discriminant(&self) -> u16 {
        unsafe { *<*const _>::from(self).cast::<u16>() }
    }
}

impl From<std::process::ExitCode> for VermanSchemaError {
    fn from(error: std::process::ExitCode) -> Self {
        Self::ExitCode(error)
    }
}

impl From<std::io::Error> for VermanSchemaError {
    fn from(error: std::io::Error) -> Self {
        Self::StdIoError { error }
    }
}

impl From<std::str::Utf8Error> for VermanSchemaError {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::Utf8Error { error }
    }
}

impl From<jaq_json::Error> for VermanSchemaError {
    fn from(error: jaq_json::Error) -> Self {
        Self::JaqJsonError { error }
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
