use clap::{Arg, ArgMatches, SubCommand};
use git_queue::queue::Queue;

use crate::{error::Error, App};

pub(super) fn subcommand() -> App {
    SubCommand::with_name("close")
        .about("Close a patch queue")
        .long_about(
            "\
Close a patch queue. You may specifcy more than one branch for deletion. \
The logs of the queues will also be deleted. If a patch is applied in the \
queue, the command will abort. The deletion can be forced using -f/--force.",
        )
        .args(&[
            Arg::with_name("force")
                .short("f")
                .long("force")
                .takes_value(false)
                .help("Force the deletion of the queues, even if they still have applied patches."),
            Arg::with_name("queue")
                .min_values(1)
                .multiple(true)
                .help("Queue to close."),
        ])
}

#[tracing::instrument(skip(args), fields(force = tracing::field::Empty, queues=tracing::field::Empty))]
pub(super) fn execute(args: &ArgMatches<'static>) -> Result<(), Error> {
    let force = args.is_present("force");
    let queues = args.values_of_lossy("queue").unwrap_or_default();

    close(queues, force)
}

fn close(queues: Vec<String>, force: bool) -> Result<(), Error> {
    let ctx = crate::git::current_git_ctx()?;

    let mut git_queues = Vec::with_capacity(queues.len());
    for q in queues {
        match Queue::for_queue(&ctx, &q) {
            Ok(Some(q)) => git_queues.push(q),
            Ok(None) => throw!(DATAERR, "Queue `{}` not found", q),
            Err(e) => return Err(e.into())
        }
    }

    for q in git_queues {
        close_one(q, force)?;
    }

    Ok(())
}

fn close_one(queue: Queue<'_>, force: bool) -> Result<(), Error> {
    if queue.is_current() {
        throw!(
            USAGE,
            "Cannot close current queue, please switch to a different queue/branch \
            before trying again."
        );
    } else if force {
        // TODO: remove any pending patches.
        queue.close()?
    } else if queue.can_close() {
        queue.close()?
    } else {
        throw!(USAGE, "The queue contains {} patches, cannot close.");
    };

    Ok(())
}
