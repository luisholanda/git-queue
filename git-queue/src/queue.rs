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
    pub fn for_branch(ctx: &'r Ctx, branch: git2::Branch<'r>) -> Result<Self, git2::Error> {
        let state = QueueState::current_for_branch(ctx.repo(), branch.name()?.unwrap())?;

        Ok(Self { branch, state, ctx })
    }

    pub fn initialize(ctx: &'r Ctx, branch: git2::Branch<'r>) -> Result<Self, git2::Error> {
        let state = QueueState::new(ctx.repo(), branch.name()?.unwrap())?;

        Ok(Self { branch, state, ctx })
    }

    pub fn new_patch(&mut self, name: String, message: String) -> Result<(), git2::Error> {
        Ok(())
    }
}
