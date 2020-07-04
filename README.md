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

In other words, if `evry` exits with a zero exit code (success), run the wget command.

`evry` exits with an unsuccessful exit code if the command has been run in the last <duration>

When `evry` exits with a successful exit code, it saves when it ran to a metadata file to `XDG_DATA_HOME/evry/data` that keep track of when some tag (e.g. -scrapesite) was last run

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

The `-runcommand` is just an arbitrary tag name so that `evry` can save metadata about a command to run/job. Can be chosen arbitrarily, its only use is to uniquely identify runs of `evry`, and save a metadata file to `XDG_DATA_HOME/evry/data`.

I have certain jobs (e.g. scraping websites for metadata, using [selenium](https://www.selenium.dev/) to login to some website and click a button, updating specific packages (e.g. `brew cask upgrade --greedy` on mac)) that I want to run periodically, but they have the chance to fail - so putting them in some script I run every so often is preferable to cron.

The duration (e.g. `evry 2 months, 5 days`) is parsed with a [`PEG`](https://en.wikipedia.org/wiki/Parsing_expression_grammar), so its very flexible. All of these are valid duration input:

* `2 months, 5 day`
* `2weeks 5hrs` (commas are optional)
* `60secs`
* `5wk, 5d`
* `5weeks, 2weeks` (is additive, so this would result in 7 weeks)
* `60sec 2weeks` (order doesn't matter)

