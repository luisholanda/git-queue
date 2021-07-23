use clap::{ArgMatches, SubCommand};
use git_queue::{ctx::Ctx, queue::Queue};

use crate::App;

pub(super) fn subcommand() -> App {
    SubCommand::with_name("init")
}

pub(super) fn execute(_: &ArgMatches<'static>) -> Result<(), ()> {
    let ctx = Ctx::current().unwrap();

    let current_branch = ctx.current_branch().unwrap();

    Queue::initialize(&ctx, current_branch).unwrap();

    Ok(())
}
