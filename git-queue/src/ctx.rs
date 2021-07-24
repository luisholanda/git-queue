use crate::gpg::GitGpg;
use git2::{build::CheckoutBuilder, Error, ErrorCode, Repository, Signature};

pub struct Ctx {
    repo: git2::Repository,
    config: git2::Config,
    user: Signature<'static>,
    gpg: GitGpg,
}

impl Ctx {
    pub fn current() -> Result<Self, Error> {
        let cwd = std::env::current_dir().map_err(|err| {
            Error::new(
                git2::ErrorCode::GenericError,
                git2::ErrorClass::Os,
                &err.to_string(),
            )
        })?;

        let repo = Repository::discover(cwd)?;
        let config = repo.config()?;
        let user = repo.signature()?.to_owned();

        let gpg = GitGpg::from_config(&config);

        Ok(Self {
            repo,
            config,
            user,
            gpg,
        })
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

    pub fn current_branch(&self) -> Result<git2::Branch<'_>, git2::Error> {
        let head = self.repo.head()?;
        if !head.is_branch() {
            return Err(git2::Error::new(
                git2::ErrorCode::Invalid,
                git2::ErrorClass::Reference,
                "current HEAD is not a branch",
            ));
        }

        Ok(git2::Branch::wrap(head))
    }

    pub fn checkout_branch(
        &self,
        branch: &git2::Branch<'_>,
        merge: bool,
    ) -> Result<(), git2::Error> {
        let tree = branch.get().peel_to_tree()?;
        self.repo.checkout_tree(
            tree.as_object(),
            Some(CheckoutBuilder::new().conflict_style_merge(merge)),
        )?;
        self.repo.set_head(branch.get().name().unwrap())
    }

    pub fn find_branch(&self, branch: &str) -> Result<Option<git2::Branch<'_>>, git2::Error> {
        match self.repo.find_reference(branch) {
            Ok(branch_ref) => Ok(Some(git2::Branch::wrap(branch_ref))),
            Err(err) if err.code() == ErrorCode::NotFound => Ok(None),
            Err(err) => Err(err),
        }
    }
}
