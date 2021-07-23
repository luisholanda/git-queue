use crate::App;
use clap::ArgMatches;

mod init;

pub(crate) type CmdExecFn = for<'a> fn(&'a ArgMatches<'static>) -> Result<(), ()>;

static EXECUTE_MAPS: phf::Map<&'static str, CmdExecFn> = phf::phf_map! {
    "init" => init::execute,
};

pub(crate) fn all() -> impl IntoIterator<Item = App> {
    [init::subcommand()]
}

pub(crate) fn get_exec_fn(subcommand: &str) -> Option<CmdExecFn> {
    EXECUTE_MAPS.get(subcommand).copied()
}
