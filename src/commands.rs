use crate::{error::Error, App};
use clap::{Arg, ArgMatches};

mod close;
mod queues;
mod switch;

pub(crate) type CmdExecFn = for<'a> fn(&'a ArgMatches<'static>) -> Result<(), Error>;

static EXECUTE_MAPS: phf::Map<&'static str, CmdExecFn> = phf::phf_map! {
    "close" => close::execute,
    "queues" => queues::execute,
    "switch" => switch::execute,
};

pub(crate) fn all() -> impl IntoIterator<Item = App> {
    [switch::subcommand(), close::subcommand(), queues::subcommand()]
}

pub(crate) fn get_exec_fn(subcommand: &str) -> Option<CmdExecFn> {
    EXECUTE_MAPS.get(subcommand).copied()
}

pub(self) fn flag(name: &'static str, short: &'static str) -> Arg<'static, 'static> {
    Arg::with_name(name)
        .short(short)
        .long(name)
        .takes_value(false)
}
