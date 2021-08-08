use git_queue::ctx::Ctx;

pub fn current_git_ctx() -> Result<Ctx, crate::error::Error> {
    match Ctx::current() {
        Ok(Some(ctx)) => Ok(ctx),
        Ok(None) => throw!(
            USAGE,
            concat!(
                "Not in Git repository! ",
                clap::crate_name!(),
                " must be executed from a non-bare Git repository"
            )
        ),
        Err(err) => crate::error::handle_any_git_error(err),
    }
}
