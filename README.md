# evry

A tool to *manually* run commands -- periodically.

### Install

Install `rust`/`cargo`, then:

```
cargo install --git https://gitlab.com/seanbreckenridge/evry
```

## Rationale

Uses shell exit codes to determine control flow in shell scripts.

```
Usage:
  evry [describe duration]... <-tagname>
  evry rollback <-tagname>
  evry help
```

Best explained with an example:

`evry 2 weeks -scrapesite && wget "https://" -o ....`

In other words, if `evry` exits runs successfully, run the `wget` command.

`evry` exits with an unsuccessful exit code if the command has been run in the last `2 weeks` (see below for more duration examples)

When `evry` exits with a successful exit code, it saves the current time to a metadata file for that tag (`-scrapsite`). That way, when `evry` is run again, it can compare the current time against that file.

Since this has no clue what the external command is, and whether it succeeds or not, this saves a history of one operation, so you can rollback when a tag was last run, incase of failure. An example:

```
evry 2 months -selenium && {
# evry succeeded, so the external command should be run
    python selenium.py || {
        # the python process exited with a non-zero exit code
        # we should rollback when the command was last run, so
        # we can re-try later
        evry rollback -selenium
    }
}
```

This can *sort of* be thought of as `cron` alternative, but operations don't run in the background. It requires you to call the command yourself, but it won't run if its already run in the time frame you describe.

You could have an infinite loop running in the background like:

```
while true; do
  evry 1 month -runcommand && run command
  sleep 60
done
```

... and even though that tries to run the command every 60 seconds, `evry` exits with an unsuccessful exit code, so `run command` would only get run once per month.

The `-runcommand` is just an arbitrary tag name so that `evry` can save metadata about a command to run/job. Can be chosen arbitrarily, its only use is to uniquely identify runs of `evry`, and save a metadata file to your [local data directory](https://docs.rs/app_dirs/1.2.1/app_dirs/)

### Duration

The duration (e.g. `evry 2 months, 5 days`) is parsed with a [`PEG`](https://en.wikipedia.org/wiki/Parsing_expression_grammar), so its very flexible. All of these are valid duration input: 
* `2 months, 5 day`
* `2weeks 5hrs` (commas are optional)
* `60secs`
* `5wk, 5d`
* `5weeks, 2weeks` (is additive, so this would result in 7 weeks)
* `60sec 2weeks` (order doesn't matter)

### Debug

The `EVRY_DEBUG` environment variable can be set to provide information on what was parsed from user input:

`EVRY_DEBUG=1 evry 3 months, 5 days -sometag`

### How I use this

I have certain jobs (e.g. scraping websites for metadata, using [selenium](https://www.selenium.dev/) to [login to some website and click a button](https://github.com/seanbreckenridge/pythonanywhere-3-months), updating specific packages (e.g. [running `brew cask upgrade --greedy` on mac](https://github.com/seanbreckenridge/dotfiles/blob/e11aea908ec4f2dd111143ebfe5d6a4eb07e268c/.config/zsh/functions/update#L11))) that I want to run periodically.

Putting all my jobs I want to run periodically in one 'housekeeping' script I run daily/weekly gives me the ability to monitor the output easily, but also allows me the flexibility of being able to schedule tasks to run at different rates.

 [anacron](https://linux.die.net/man/8/anacron) tries to solve a similar problem, and I use both `cron` and `anacron` for different uses. I prefer `evry`s way of describing the duration, and `anacron` is typically run with a `systemd` timer anyways. `evry` is *not meant* to be run as a daemon, its specifically for tasks I want to monitor manually.
