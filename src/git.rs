use git_queue::ctx::Ctx;

pub fn current_git_ctx() -> Result<Ctx, crate::error::Error> {
    if let Some(ctx) = Ctx::current()? {
        Ok(ctx)
    } else {
        throw!(
            USAGE,
            concat!(
                "Not in Git repository! ",
                clap::crate_name!(),
                " must be executed from a non-bare Git repository"
            )
        )
    }
}
