#![warn(missing_docs)]
#![warn(missing_doc_code_examples)]
#![allow(clippy::needless_return)]

//! A shell-script-centric task scheduler; uses exit codes to determine control flow.
//!
//! Best explained with an example:
//!
//! `evry 2 weeks -scrapesite && wget "https://" -o ....`
//!
//! In other words, run the `wget` command every `2 weeks`.
//!
//! `evry` exits with an unsuccessful exit code if the command has been run in the last `2 weeks` (see the parser module for more examples), which means the `wget` command wouldn't run.
//!
//! When `evry` exits with a successful exit code, it saves the current time to a metadata file for that tag (`-scrapesite`). That way, when `evry` is run again with that tag, it can compare the current time against that file.
//!
//! This can *sort of* be thought of as `cron` alternative, but operations don't run in the background. It requires you to call the command yourself, but it won't run if its already run in the time frame you describe.
//!
//! You could have an infinite loop running in the background like:
//!
//! ```bash
//! while true; do
//!   evry 1 month -runcommand && run command
//!   sleep 60
//! done
//! ```
//!
//! ... and even though that tries to run the command every 60 seconds, `evry` exits with an unsuccessful exit code, so `run command` would only get run once per month.
//!
//! The `-runcommand` is just an arbitrary tag name so that `evry` can save metadata about a command to run/job. Can be chosen arbitrarily, its only use is to uniquely identify runs of `evry`, and save a metadata file to your [local data directory](https://docs.rs/app_dirs/1.2.1/app_dirs/)
//!
//! Since this doesn't run in a larger context and its just a bash script, if a command fails, you can remove the tag file, to reset it to run again later (since if the file doesn't exist, `evry` assumes its a new task)

use std::env;
use std::process::exit;

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod file;
mod parser;
mod printer;
mod utils;

/// parses the user input; flags/environment variables
#[derive(Debug)]
struct Args {
    /// unparsed, string representation of a date from the user
    raw_date: String,
    /// if EVRY_DEBUG=1 was set
    debug: bool,
    /// if EVRY_JSON=1 was set
    json: bool,
    /// if the user asked to print the computed tag location instead of running
    location: bool,
    /// tagfile to read/write from, uniquely identifies this job
    tag: file::Tag,
}

impl Args {
    /// prints the help message
    fn help(warn: bool) {
        if warn {
            println!("Not enough arguments provided.");
        }
        println!(
            "A tool to manually run commands -- periodically.
Uses shell exit codes to determine control flow in shell scripts

Usage:
  evry [describe duration]... <-tagname>
  evry location <-tagname>
  evry help

Best explained with an example:

evry 2 weeks -scrapesite && wget \"https://\" -o ....

In other words, run the wget command every 2 weeks.

evry exits with an unsuccessful exit code if the command has
been run in the last 2 weeks, which means the wget command wouldn't run.

When evry exits with a successful exit code, it saves the current time
to a metadata file for that tag (-scrapesite). That way, when evry
is run again with that tag, it can compare the current time against that file.

location prints the computed tag file location

See https://github.com/seanbreckenridge/evry for more examples."
        );
        exit(10)
    }

    /// parses command-line user input/environment variables
    fn parse_args(dir_info: &file::LocalDir) -> Self {
        // get arguments (remove binary name)
        let args: Vec<String> = env::args().skip(1).collect();
        let args_len = args.len();
        // if user asked for help
        if args_len >= 1 && (args[0] == "help" || args[0] == "--help") {
            Args::help(false)
        }
        // split args arguments into tag/other strings
        let (tag_vec, other_vec): (_, Vec<_>) =
            args.into_iter().partition(|arg| arg.starts_with('-'));
        // user didn't provide argument
        if tag_vec.is_empty() || other_vec.is_empty() {
            Args::help(true)
        }
        // parse tag, remove the '-' from the name
        let tag_raw = &tag_vec[0];
        let tag = tag_raw
            .chars()
            .next()
            .map(|c| &tag_raw[c.len_utf8()..])
            .expect("Error: Couldn't parse tag from arguments");
        if tag.chars().count() == 0 {
            eprintln!("Error: passed tag was an empty string");
        }
        let first_arg = &other_vec[0];
        let json = env::var("EVRY_JSON").is_ok();
        Args {
            raw_date: other_vec.join(" "),
            // specifying EVRY_JSON automatically enables debug as well
            // otherwise evry is supposed to remain silent -- its not meant to print anything
            debug: json | env::var("EVRY_DEBUG").is_ok(),
            json,
            location: first_arg == "location",
            tag: file::Tag::new(tag.to_string(), dir_info),
        }
    }
}

/// encapsulates the logic for evry, printing logs to the printer
/// if debug is enabled.
/// Returns an exit code to signify what to do
fn evry(dir_info: file::LocalDir, cli: Args, printer: &mut printer::Printer) -> i32 {
    if cli.debug {
        printer.echo("tag_name", &cli.tag.name);
        let mut d = dir_info.root_dir;
        d.push("data");
        printer.echo("data_directory", &d.into_os_string().into_string().unwrap());
    }

    if cli.location {
        // causes an early exit, print directly instead of using the printer
        // user is probably trying to use this to compute the location like
        // SHELLVAR="$(evry location -tagname)"
        println!("{}", cli.tag.path);
        return 0;
    }

    // parse duration string
    let run_every = match parser::parse_time(&cli.raw_date) {
        Ok(time) => time,
        Err(_e) => {
            printer.echo(
                "error",
                &format!("couldn't parse '{}' into a duration", cli.raw_date),
            );
            // eprintln!("{:?}", _e);
            return 1; // signify fatal error
        }
    };

    // get current time
    let now = utils::epoch_millis();

    if cli.debug {
        printer.echo(
            "log",
            &format!("parsed '{}' into {}ms", cli.raw_date, run_every),
        );
        printer.print(
            printer::Message::new("duration", &format!("{}", run_every)),
            Some(printer::PrinterType::Json),
        );
        printer.print(
            printer::Message::new("duration_pretty", &utils::describe_ms(run_every)),
            Some(printer::PrinterType::Json),
        );
    }

    if !cli.tag.file_exists() {
        // file doesn't exist, this is the first time this tag is being run.
        // save the current milliseconds to the file and exit with a 0 exit code
        if cli.debug {
            printer.echo(
                "log",
                "Tag file doesn't exist, creating and exiting with code 0",
            );
        }
        cli.tag.write(now);
        return 0;
    } else {
        // file exists, read last time this tag was run
        let last_ran_at = cli.tag.read_epoch_millis();
        if now - last_ran_at > run_every {
            // duration this should be run at has elapsed, run
            if cli.debug {
                printer.echo("log", &format!("Has been more than '{}' ({}ms) since last succeeded, writing to tag file, exiting with code 0", utils::describe_ms(run_every), run_every));
            }
            // save current time to tag file
            cli.tag.write(now);
            return 0;
        } else {
            // this has been run within the specified duration, don't run
            if cli.debug {
                printer.echo(
                    "log",
                    &format!(
                        "{} ({}ms) haven't elapsed since last run, exiting with code 1",
                        utils::describe_ms(run_every),
                        run_every
                    ),
                );
                let till_next_run = last_ran_at + run_every - now;
                let till_next_pretty = utils::describe_ms(till_next_run);
                printer.echo(
                    "log",
                    &format!(
                        "Will next be able to run in '{}' ({}ms)",
                        till_next_pretty, till_next_run
                    ),
                );
                printer.print(
                    printer::Message::new("till_next", &format!("{}", till_next_run)),
                    Some(printer::PrinterType::Json),
                );
                printer.print(
                    printer::Message::new("till_next_pretty", &till_next_pretty),
                    Some(printer::PrinterType::Json),
                );
            }
            return 2; // exit code 2; expected error, to cause next shell command to not run
        }
    }
}

fn main() {
    // global application information
    let dir_info = file::LocalDir::new();
    let cli = Args::parse_args(&dir_info);

    let printer_type = if cli.json {
        printer::PrinterType::Json
    } else {
        printer::PrinterType::Stderr
    };

    // handles printing/saving messages incase we're in JSON mode
    let mut printer = printer::Printer::new(printer_type);

    // run 'main' code, saving exit code
    let result = evry(dir_info, cli, &mut printer);

    // if user specified JSON, print the blob
    printer.flush();
    exit(result);
}
