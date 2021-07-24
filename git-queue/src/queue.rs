use crate::ctx::Ctx;
use self::log::QueueState;

mod log;
pub mod patch;

pub struct Queue<'r> {
    branch: git2::Branch<'r>,
    state: QueueState,
    ctx: &'r Ctx
}

impl<'r> Queue<'r> {
    pub fn for_queue(ctx: &'r Ctx, queue: &str) -> Result<Self, git2::Error> {
        let branch = ctx.repo().find_branch(&Self::gitref_name(queue), git2::BranchType::Local)?;
        let state = QueueState::current_for_queue(ctx.repo(), queue)?;

        Ok(Self { branch, state, ctx })
    }

    pub fn initialize(ctx: &'r Ctx, name: &str, branch: git2::Branch<'r>) -> Result<Self, git2::Error> {
        let base = branch.get().peel_to_commit()?;
        let queue_branch = ctx.repo().branch(&Self::gitref_name(name), &base, false)?;
        let state = QueueState::new(ctx.repo(), name, &branch)?;

        Ok(Self { branch: queue_branch, state, ctx })
    }

    pub fn switch_to(&self, merge: bool) -> Result<(), git2::Error> {
        self.ctx.checkout_branch(&self.branch, merge)
    }

    fn gitref_name(queue: &str) -> String {
        format!("queues/{}", queue)
    }
}
