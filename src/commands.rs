use crate::{error::Error, App};
use clap::ArgMatches;

mod close;
mod switch;

pub(crate) type CmdExecFn = for<'a> fn(&'a ArgMatches<'static>) -> Result<(), Error>;

static EXECUTE_MAPS: phf::Map<&'static str, CmdExecFn> = phf::phf_map! {
    "close" => close::execute,
    "switch" => switch::execute,
};

pub(crate) fn all() -> impl IntoIterator<Item = App> {
    [switch::subcommand(), close::subcommand()]
}

pub(crate) fn get_exec_fn(subcommand: &str) -> Option<CmdExecFn> {
    EXECUTE_MAPS.get(subcommand).copied()
}
