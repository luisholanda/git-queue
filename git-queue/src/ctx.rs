use crate::error::Error;
use crate::gpg::GitGpg;
use git2::{build::CheckoutBuilder, ErrorClass, ErrorCode, Repository, Signature};

pub struct Ctx {
    repo: git2::Repository,
    config: git2::Config,
    user: Signature<'static>,
    gpg: GitGpg,
}

impl Ctx {
    pub fn current() -> Result<Option<Self>, Error> {
        let cwd = std::env::current_dir().map_err(|err| {
            Error::Git(git2::Error::new(
                git2::ErrorCode::GenericError,
                git2::ErrorClass::Os,
                &err.to_string(),
            ))
        })?;

        let repo = match Repository::discover(cwd) {
            Ok(repo) => repo,
            Err(e) if e.class() == ErrorClass::Repository && e.code() == ErrorCode::NotFound => {
                return Ok(None)
            }
            Err(err) => return Err(err.into()),
        };
        let config = repo.config()?;
        let user = repo.signature()?.to_owned();

        let gpg = GitGpg::from_config(&config);

        Ok(Some(Self {
            repo,
            config,
            user,
            gpg,
        }))
    }

    pub const fn repo(&self) -> &git2::Repository {
        &self.repo
    }

    pub const fn config(&self) -> &git2::Config {
        &self.config
    }

    pub const fn user(&self) -> &git2::Signature<'static> {
        &self.user
    }

    pub fn current_branch(&self) -> Result<Option<git2::Branch<'_>>, Error> {
        let head = match self.repo.head() {
            Ok(h) => h,
            Err(err) if matches!(err.code(), ErrorCode::NotFound | ErrorCode::UnbornBranch) => {
                return Ok(None)
            }
            Err(err) => return Err(err.into()),
        };
        if !head.is_branch() {
            return Err(git2::Error::new(
                git2::ErrorCode::Invalid,
                git2::ErrorClass::Reference,
                "current HEAD is not a branch",
            )
            .into());
        }

        Ok(Some(git2::Branch::wrap(head)))
    }

    pub fn checkout_branch(&self, branch: &git2::Branch<'_>, merge: bool) -> Result<(), Error> {
        let tree = branch.get().peel_to_tree()?;
        self.repo.checkout_tree(
            tree.as_object(),
            Some(CheckoutBuilder::new().conflict_style_merge(merge)),
        )?;
        let name = branch.get().name().ok_or(Error::NonUtf8)?;
        Ok(self.repo.set_head(name)?)
    }

    pub fn find_branch(&self, branch: &str) -> Result<Option<git2::Branch<'_>>, Error> {
        match self.repo.find_reference(branch) {
            Ok(branch_ref) => Ok(Some(git2::Branch::wrap(branch_ref))),
            Err(err) if err.code() == ErrorCode::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}
