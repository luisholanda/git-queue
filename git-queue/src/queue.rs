use git2::{BranchType, ErrorCode};

use self::log::QueueState;
use crate::{ctx::Ctx, error::Error};

mod log;
pub mod patch;

pub struct Queue<'r> {
    branch: git2::Branch<'r>,
    state: QueueState,
    ctx: &'r Ctx,
}

impl<'r> Queue<'r> {
    pub fn for_queue(ctx: &'r Ctx, queue: &str) -> Result<Option<Self>, Error> {
        let branch = match ctx
            .repo()
            .find_branch(&Self::gitref_name(queue), git2::BranchType::Local)
        {
            Ok(branch) => branch,
            Err(err) if err.code() == ErrorCode::NotFound => return Ok(None),
            Err(err) if err.code() == ErrorCode::InvalidSpec => return Err(Error::InvalidName),
            Err(err) => return Err(err.into()),
        };
        let state = QueueState::current_for_queue(ctx.repo(), queue)?;

        Ok(Some(Self { branch, state, ctx }))
    }

    pub fn current(ctx: &'r Ctx) -> Result<Option<Self>, Error> {
        let branch = if let Some(branch) = ctx.current_branch()? {
            branch
        } else {
            // If there is no current branch, there is no current queue.
            return Ok(None);
        };
        let name = branch.name()?.unwrap();
        if let Some((_, queue)) = name.split_once("queues/") {
            Self::for_queue(ctx, queue)
        } else {
            Ok(None)
        }
    }

    pub fn initialize(
        ctx: &'r Ctx,
        name: &str,
        branch: git2::Branch<'r>,
    ) -> Result<Option<Self>, Error> {
        let base = branch.get().peel_to_commit()?;
        let queue_branch = match ctx.repo().branch(&Self::gitref_name(name), &base, false) {
            Ok(b) => b,
            Err(err) if err.code() == ErrorCode::Exists => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        let state = QueueState::new(ctx.repo(), name, &branch)?;

        Ok(Some(Self {
            branch: queue_branch,
            state,
            ctx,
        }))
    }

    pub fn list(ctx: &'r Ctx) -> Result<impl Iterator<Item = Result<Queue<'r>, Error>>, Error> {
        let branches = ctx.repo().branches(Some(BranchType::Local))?;

        Ok(branches.filter_map(move |r| {
            r.map_err(Error::Git)
                .and_then(|(b, _)| {
                    let name = b.name()?.ok_or(Error::NonUtf8)?;
                    if name.starts_with("queues/") {
                        let name = name.split('/').nth(1).unwrap();
                        Self::for_queue(ctx, name)?
                            .ok_or_else(|| Error::Inconsistency("queuelog"))
                            .map(Some)
                    } else {
                        Ok(None)
                    }
                })
                .transpose()
        }))
    }

    pub fn name(&self) -> &str {
        self.state.name()
    }

    pub fn base_name(&self) -> &str {
        self.state.base_name()
    }

    pub fn can_close(&self) -> bool {
        self.state.patches_num() == 0
    }

    pub fn is_current(&self) -> bool {
        self.branch.is_head()
    }

    pub fn switch_to(&self, merge: bool) -> Result<(), Error> {
        self.ctx.checkout_branch(&self.branch, merge)
    }

    pub fn close(mut self) -> Result<(), Error> {
        assert!(!self.is_current(), "tried to close current queue");
        assert_eq!(
            self.state.patches_num(),
            0,
            "tried to close queue with associated patches"
        );

        self.branch.delete()?;
        let find_ref_res = self.ctx.repo().find_reference(self.state.gitref());

        match find_ref_res {
            Ok(mut git_ref) => Ok(git_ref.delete()?),
            // Ref was already deleted, maybe manually?
            Err(err) if err.code() == git2::ErrorCode::NotFound => {
                tracing::warn!("reference `{}` was already deleted!", self.state.gitref());
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }

    fn gitref_name(queue: &str) -> String {
        format!("queues/{}", queue)
    }
}
