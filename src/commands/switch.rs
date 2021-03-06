use clap::{Arg, ArgMatches, SubCommand};
use git_queue::queue::Queue;

use crate::{error::Error, App};

pub(super) fn subcommand() -> App {
    SubCommand::with_name("switch")
        .about("Switch queues")
        .long_about(
            "\
Switch to a specified queue. The working tree and the index are \
updated to match the applied patches in the queue. All new patches will be \
added to the top of this queue.

Optionally a new queue could be created with --create, along with switching. \
By default, the queue will be created using the same base used for the current \
queue, if you are not in a queue, the default value will be the current branch. \
Both of these can be overwritten passing the desired branch in <branch>.

Switching queues does not require a clean index and working tree. The \
operation is aborted however if the operation leads to conflicts.",
        )
        .args(&[
            super::flag("create", "c")
                .help("Create a new queue with name given by <queue>."),
            Arg::with_name("queue")
                .required(true)
                .empty_values(false)
                .help("Queue to switch to."),
            Arg::with_name("branch")
                .required(false)
                .empty_values(false)
                .help("Branch to use as base of the created queue."),
        ])
}

#[tracing::instrument(skip(args), fields(
        queue = tracing::field::Empty,
        create = tracing::field::Empty,
        branch = tracing::field::Empty))]
pub(super) fn execute(args: &ArgMatches<'static>) -> Result<(), Error> {
    let queue = args
        .value_of("queue")
        .expect("Missing required <queue> parameter");
    let create = args.is_present("create");
    let branch = args.value_of("branch");

    tracing::Span::current()
        .record("queue", &queue)
        .record("create", &create)
        .record("branch", &tracing::field::debug(branch));

    switch(queue, create, branch)
}

fn switch(queue: &str, create: bool, branch: Option<&str>) -> Result<(), Error> {
    let ctx = crate::git::current_git_ctx()?;

    let queue = match Queue::for_queue(&ctx, queue) {
        Ok(Some(queue)) => queue,
        Ok(None) => {
            if !create {
                throw!(DATAERR, "Queue `{}` does not exist", queue);
            }

            let base_branch = if let Some(branch) = branch {
                match ctx.find_branch(branch) {
                    Ok(Some(branch)) => branch,
                    Ok(None) => throw!(DATAERR, "Branch {} does not exist", branch),
                    Err(err) => return Err(err.into())
                }
            } else if let Some(branch) = ctx.current_branch()? {
                branch
            } else {
                crate::error::not_properly_initialized()?
            };

            // We did just check that the queue didn't exist, so this cannot return Ok(None).
            Queue::initialize(&ctx, queue, base_branch)?.unwrap()
        }
        Err(err) => return Err(err.into())
    };

    queue.switch_to()?;

    let statuses = ctx.workdir_status()?;
    for status in statuses.iter() {
        if status.status().is_conflicted() {
            if let Some(fp) = status.path() {
                eprintln!("{} has conflicts after switching to the queue.", fp);
            }
        }
    }

    Ok(())
}
