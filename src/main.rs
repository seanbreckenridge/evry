use std::env;
use std::process::exit;

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod app_path;
mod parser;

#[derive(Debug)]
pub struct CLI {
    args: Vec<String>,
    debug: bool,
    rollback: bool,
    tag: app_path::Tag,
}

impl CLI {
    fn help(warn: bool) {
        if warn {
            println!("Not enough arguments provided.");
        }
        println!(
            "A tool to manually run commands -- periodically.
Uses shell exit codes to determine control flow

Usage:
  evry [describe duration]... <-tagname>
  evry rollback <-tagname>
  evry help

Best explained with an example:

evry 2 weeks -scrapesite && wget \"https://\" -o ....

In other words, if evry exits with a zero exit code (success),
run the wget command.

evry exits with an unsuccessful exit code
if the command has been run in the last <duration>

This saves files to XDG_DATA_HOME/evry/data that keep
track of when some tag (e.g. -scrapesite) was last run

Since this has no clue what the external command is,
and whether it succeeds or not, this saves a history
of one operation, so you can rollback when a tag was
last run, incase of failure. An example:

evry 2 months -selenium && {{
    python selenium.py || {{
        evry rollback -selenium
    }}
}}

See https://github.com/seanbreckenridge/evry for more examples.
"
        );
        exit(2)
    }

    pub fn parse_args(dir_info: &app_path::LocalDir) -> Self {
        // get arguments (remove binary name)
        let args: Vec<String> = env::args().skip(1).collect();
        let args_len = args.len();
        // if user asked for help
        if args_len >= 1 && (args[0] == "help" || args[0] == "--help") {
            CLI::help(false)
        }
        // split CLI arguments into tag/other strings
        let (tag_vec, other_vec): (_, Vec<_>) =
            args.into_iter().partition(|arg| arg.starts_with('-'));
        // user didn't provide argument
        if tag_vec.is_empty() || other_vec.is_empty() {
            CLI::help(true)
        }
        // parse tag, remove the '-' from the name
        let tag_raw = &tag_vec[0];
        let tag = tag_raw
            .chars()
            .next()
            .map(|c| &tag_raw[c.len_utf8()..])
            .expect("Couldn't parse tag from arguments");
        // if asked to rollback
        let rollback = &other_vec[0] == "rollback";
        CLI {
            args: other_vec,
            debug: env::var("EVRY_DEBUG").is_ok(),
            rollback,
            tag: app_path::Tag::new(tag.to_string(), dir_info),
        }
    }
}

fn main() {
    // global application information
    let dir_info = app_path::LocalDir::new();
    let cli = CLI::parse_args(&dir_info);
    if cli.debug {
        println!("{:?}", dir_info);
        println!("{:?}", cli);
    }

    if cli.rollback {
        if cli.debug {
            println!("Running rollback...");
        }
        app_path::restore_rollback(&dir_info, &cli.tag);
        exit(0)
    }

    // parse duration string
    let run_every = parser::parse_time(cli.args, cli.debug);

    if cli.debug {
        println!("Parsed input date string into {} milliseconds", run_every);
    }

    if !cli.tag.file_exists() {
        // file doesn't exist, this is the first time this tag is being run.
        // save the current milliseconds to the file and exit with a 0 exit code
        cli.tag.write_epoch_millis();
        exit(0)
    } else {
        let epoch_now = app_path::epoch_time();
        // file exists, read last time this tag was run
        let last_ran_at = cli.tag.read_epoch_millis();
        if epoch_now - last_ran_at > run_every {
            // duration this should be run at has elapsed, run
            if cli.debug {
                println!("Has been more than {:?} milliseconds, saving to rollback file and writing to tag file", run_every)
            }
            // dump this to rollback file so it can this can be rolled back if external command fails
            app_path::save_rollback(&dir_info, last_ran_at);
            // save current time to tag file
            cli.tag.write_epoch_millis();
            exit(0)
        } else {
            // this has been run within the specified duration, don't run
            exit(1)
        }
    }
}
