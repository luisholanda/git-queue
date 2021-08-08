#[derive(Debug)]
pub enum Error {
    NotInRepository,
    NotInitialized,
    Inconsistency(&'static str),
    InvalidName,
    AlreadyExists(&'static str),
    NonUtf8,
    Git(git2::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInRepository => f.write_str("not in a repository"),
            Self::NotInitialized => f.write_str("the repository exist but is not initialized"),
            Self::Inconsistency(i) => write!(
                f,
                "detected inconsistency in {}, did you run a git command manually?",
                i
            ),
            Self::InvalidName => f.write_str("the received name is invalid"),
            Self::NonUtf8 => f.write_str("the received name is not valid UTF-8"),
            Self::AlreadyExists(b) => write!(f, "{} already exists", b),
            Self::Git(g) => g.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Self::Git(g) = self {
            Some(g)
        } else {
            None
        }
    }
}

impl From<git2::Error> for Error {
    fn from(err: git2::Error) -> Self {
        Self::Git(err)
    }
}
