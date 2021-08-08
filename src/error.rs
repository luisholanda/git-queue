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

#[macro_export]
macro_rules! throw {
    ($code: ident, $($t: tt)+) => {
        return Err($crate::error::Error::new(::exitcode::$code, ::anyhow::anyhow!($($t)+)));
    }
}

pub fn handle_any_git_error<T>(err: git_queue::Error) -> Result<T, Error> {
    use git_queue::Error::*;
    match err {
        err @ (NotInRepository | NotInitialized) => throw!(USAGE, err),
        err @ (Inconsistency(_) | InvalidName | NonUtf8) => throw!(DATAERR, err),
        err @ AlreadyExists(_) => throw!(CANTCREAT, err),
        Git(err) => match err.class() {
            ErrorClass::Reference if err.code() == ErrorCode::UnbornBranch => {
                throw!(USAGE, "The current branch is not initialized")
            }
            ErrorClass::Os => throw!(OSERR, err),
            ErrorClass::Filesystem | ErrorClass::Net => throw!(IOERR, err),
            ErrorClass::NoMemory => throw!(TEMPFAIL, err),
            _ => unreachable!("Internal error. This is a bug!"),
        },
    }
}

pub fn not_properly_initialized<T>() -> Result<T, Error> {
    throw!(USAGE, "Properly initialize your repository and try again")
}

#[macro_export]
macro_rules! ensure {
    ($exp: expr) => {
        match $exp {
            Ok(ret) => ret,
            Err(err) => $crate::error::handle_any_git_error(err)?,
        }
    };
}
