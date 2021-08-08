use clap::{ArgMatches, SubCommand};
use git_queue::queue::Queue;
use prettytable::{Attr, Cell, Row, Table, color::BRIGHT_GREEN};

use crate::{error::Error, App};

pub(super) fn subcommand() -> App {
    SubCommand::with_name("queues")
        .about("List available queues")
        .long_about(
            "\
List all available queues, showing individual information about each queue.

The queues will be printed in a table, showing first the name (in green if it is \
the current queue), followed by the base (if -B/--no-base is not specified).
",
        )
        .args(&[
            super::flag("no-base", "B").help("Do not show the base for each queue"),
            super::flag("no-patches", "P").help("Do not show description of patches for each queue"),
        ])
}

#[tracing::instrument(skip(args), fields(
    base = tracing::field::Empty,
    patches = tracing::field::Empty,
))]
pub(super) fn execute(args: &ArgMatches<'static>) -> Result<(), Error> {
    let base = !args.is_present("no-base");
    let patches = !args.is_present("no-patches");

    tracing::Span::current()
        .record("base", &base)
        .record("patches", &patches);

    let ctx = crate::git::current_git_ctx()?;

    let mut queues = Queue::list(&ctx)?;

    if base || patches {
        let mut titles = vec!["Name"];
        if base {
            titles.push("Base");
        }
        if patches {
            titles.push("Patches");
            titles.push("Last patch");
        }

        let mut table = crate::table::new(titles.into_iter());
        while let Some(q) = queues.next().transpose()? {
            print_queue(q, &mut table, base, patches);
        }

        table.printstd();
    } else {
        while let Some(q) = queues.next().transpose()? {
            println!("{}", q.name());
        }
    }

    Ok(())
}

fn print_queue(
    q: Queue<'_>,
    table: &mut Table,
    base: bool,
    patches: bool,
) {
    let mut name_cell = Cell::new(q.name());
    if q.is_current() {
        name_cell.style(Attr::ForegroundColor(BRIGHT_GREEN));
    }
    let mut row = Row::new(vec![name_cell]);

    if base {
        row.add_cell(Cell::new(q.base_name()));
    }

    table.add_row(row);
}
