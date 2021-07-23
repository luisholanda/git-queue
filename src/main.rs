#[macro_use]
extern crate clap;
use clap::{Arg, Shell, SubCommand};

mod commands;

pub(crate) type App = clap::App<'static, 'static>;

const GENERATE_COMPLETIONS: &'static str = "generate-completions";

fn build_app() -> App {
    clap::app_from_crate!()
        .subcommands(crate::commands::all())
        .subcommand(
            SubCommand::with_name(GENERATE_COMPLETIONS)
                .arg(
                    Arg::with_name("shell")
                        .required(true)
                        .index(1)
                        .possible_values(&Shell::variants())
                        .help("Shell to generate completions for."),
                )
                .about("Generate shell completions for a specific shell.")
                .long_about(
                    "Generate shell completions for a specific shell.

The completions are written to the standard output, redirect to a file to persist it.",
                ),
        )
}

fn generate_completions(matches: &clap::ArgMatches<'_>) {
    let shell: Shell = matches
        .value_of("shell")
        .expect("Missing required `shell` argument")
        .parse()
        .expect("Invalid value for `shell` argument");

    build_app().gen_completions_to(clap::crate_name!(), shell, &mut std::io::stdout());
}

fn main() {
    let app = build_app();
    let matches = app.get_matches();

    if let (subcmd, Some(submatches)) = matches.subcommand() {
        if let Some(execfn) = commands::get_exec_fn(subcmd) {
            let _ = execfn(submatches);
        } else if subcmd == GENERATE_COMPLETIONS {
            generate_completions(submatches);
        }
    } else {
        std::process::exit(1);
    }
}
