use exitcode::ExitCode;
use git_queue::{ErrorClass, ErrorCode};

#[derive(Debug)]
pub struct Error {
    inner: anyhow::Error,
    code: ExitCode,
}

impl Error {
    pub fn new(code: ExitCode, inner: anyhow::Error) -> Self {
        Self { inner, code }
    }

    pub fn report<W: std::io::Write>(self, writer: &mut W) -> ! {
        if writeln!(writer, "{}", self).is_ok() {
            let _ = writer.flush();
        }
        std::process::exit(self.code);
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl From<git_queue::Error> for Error {
    fn from(err: git_queue::Error) -> Self {
        use git_queue::Error::*;
        let code = match &err {
            NotInRepository | NotInitialized => exitcode::USAGE,
            Inconsistency(_) | InvalidName | NonUtf8 => exitcode::DATAERR,
            AlreadyExists(_) => exitcode::CANTCREAT,
            Git(err) => match err.class() {
                ErrorClass::Reference if err.code() == ErrorCode::UnbornBranch => {
                    return Error::new(
                        exitcode::USAGE,
                        anyhow::anyhow!("The current branch is not initialized"),
                    )
                }
                ErrorClass::Os => exitcode::OSERR,
                ErrorClass::Filesystem | ErrorClass::Net => exitcode::IOERR,
                ErrorClass::NoMemory => exitcode::TEMPFAIL,
                _ => panic!("Internal error. This is a bug! Error: {}", err),
            },
        };

        Error::new(code, anyhow::anyhow!(err))
    }
}

#[macro_export]
macro_rules! throw {
    ($code: ident, $($t: tt)+) => {
        return Err($crate::error::Error::new(::exitcode::$code, ::anyhow::anyhow!($($t)+)));
    }
}

pub fn not_properly_initialized<T>() -> Result<T, Error> {
    throw!(USAGE, "Properly initialize your repository and try again")
}
