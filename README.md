# evry

A shell-script-centric task scheduler; uses exit codes to determine control flow. Most of the time I call this behind [bgproc](https://github.com/seanbreckenridge/bgproc).

### Install

Install `rust`/`cargo`, then:

```
cargo install evry
```

## Rationale

```
A tool to manually run commands -- periodically.
Uses shell exit codes to determine control flow in shell scripts

Usage:
  evry [describe duration]... <-tagname>
  evry location <-tagname>
  evry help
```

Best explained with an example:

`evry 2 weeks -scrapesite && wget "https://" -o ....`

In other words, run the `wget` command every `2 weeks`.

`evry` exits with an unsuccessful exit code if the command has been run in the last `2 weeks` (see below for more duration examples), which means the `wget` command wouldn't run.

When `evry` exits with a successful exit code, it saves the current time to a metadata file for that tag (`-scrapesite`). That way, when `evry` is run again with that tag, it can compare the current time against that file.

This can _sort of_ be thought of as `cron` alternative, but operations don't run in the background. It requires you to call the command yourself, but it won't run if its already run in the time frame you describe. (However, its not difficult to wrap tasks that run behind `evry` in an infinite loop that runs in the background, which is what [`bgproc`](https://github.com/seanbreckenridge/bgproc) does)

You could have an infinite loop running in the background like:

```bash
while true; do
  evry 1 month -runcommand && run command
  sleep 60
done
```

... and even though that tries to run the command every 60 seconds, `evry` exits with an unsuccessful exit code, so `run command` would only get run once per month.

The `-runcommand` is just an arbitrary tag name so that `evry` can save metadata about a command to run/job. It can be chosen arbitrarily, its only use is to uniquely identify some task, and save a metadata file to your [local data directory](https://docs.rs/app_dirs/1.2.1/app_dirs/).

Since this doesn't run in a larger context and `evry` can't know if a command failed to run - if a command fails, you can remove the tag file, to reset it to run again later (since if the file doesn't exist, `evry` assumes its a new task):

```bash
evry 2 months -selenium && {
# evry succeeded, so the external command should be run
    python selenium.py || {
        # the python process exited with a non-zero exit code
        # remove the tag file so we can re-try later
        rm "$(evry location -selenium)"
        # maybe notify you that this failed so you go and check on it
        notify-send -u critical 'selenium failed!"
    }
}
```

### Duration

The duration (e.g. `evry 2 months, 5 days`) is parsed with a [`PEG`](https://en.wikipedia.org/wiki/Parsing_expression_grammar), so its very flexible. All of these are valid duration input:

- `2 months, 5 day`
- `2weeks 5hrs` (commas are optional)
- `60secs`
- `5wk, 5d`
- `5weeks, 2weeks` (is additive, so this would result in 7 weeks)
- `60sec 2weeks` (order doesn't matter)

See [the grammar](https://github.com/seanbreckenridge/evry/blob/f158ecb3632930d2b71d597bdfbd6a6e7ee16104/src/time.pest#L5-L11) for all possible abbreviations.

### Advanced Usage

The `EVRY_DEBUG` environment variable can be set to provide information on what was parsed from user input, and how long till the next run succeeds.

```
$ EVRY_DEBUG=1 evry 2 months -pythonanywhere && pythonanywhere_3_months -Hc "$(which chromedriver)"
tag_name:pythonanywhere
data_directory:/home/sean/.local/share/evry/data
log:parsed '2 months' into 5184000000ms
log:60 days (5184000000ms) haven't elapsed since last run, exiting with code 1
log:Will next be able to run in '46 days, 16 hours, 46 minutes, 6 seconds' (4034766587ms)
```

If you wanted to 'reset' a task, you could do: `rm ~/.local/share/evry/data/<tag name>`; removing the tag file. The next time that `evry` runs, it'll assume its a new task, and exit successfully. I use the following shell function to 'reset' tasks:

```bash
job-reset () {
        local EVRY_DATA_DIR CHOSEN_TAG
        # use the 'location' command with an arbitrary tag to get the data dir
        EVRY_DATA_DIR="$(dirname "$(evry location -tag)")"
        cd "${EVRY_DATA_DIR}"
        CHOSEN_TAG="$(fzf)"  || return 1
        rm -v "${CHOSEN_TAG}"
        cd -
}
```

The `EVRY_JSON` environment variable can be set to provide similar information in a more consumable format (e.g. with [`jq`](https://github.com/stedolan/jq))

As an example; `./schedule_task`:

```bash
#!/bin/bash

if JSON_OUTPUT="$(EVRY_JSON=1 evry 2 hours -task)"; then
  echo "Running task..."
else
  # extract the body for a particular log message
  NEXT_RUN="$(echo "$JSON_OUTPUT" | jq -r '.[] | select(.type == "till_next_pretty") | .body')"
  printf 'task will next run in %s\n' "$NEXT_RUN"
fi
```

```
$ ./schedule_task
Running task...
$ ./schedule_task
task will next run in 1 hours, 59 minutes, 58 seconds
```

For reference, typical JSON output when `evry` fails (command doesn't run):

```json
[
  {
    "type": "tag_name",
    "body": "task"
  },
  {
    "type": "data_directory",
    "body": "/home/sean/.local/share/evry/data"
  },
  {
    "type": "log",
    "body": "parsed '2 hours' into 7200000ms"
  },
  {
    "type": "duration",
    "body": "7200000"
  },
  {
    "type": "duration_pretty",
    "body": "2 hours"
  },
  {
    "type": "log",
    "body": "2 hours (7200000ms) haven't elapsed since last run, exiting with code 1"
  },
  {
    "type": "log",
    "body": "Will next be able to run in '1 hours, 58 minutes, 17 seconds' (7097748ms)"
  },
  {
    "type": "till_next",
    "body": "7097748"
  },
  {
    "type": "till_next_pretty",
    "body": "1 hours, 58 minutes, 17 seconds"
  }
]
```

### How I use this

I have certain jobs (e.g. scraping websites for metadata, using [selenium](https://www.selenium.dev/) to [login to some website and click a button](https://github.com/seanbreckenridge/pythonanywhere-3-months), updating specific packages (e.g. [running `brew cask upgrade --greedy` on mac](https://github.com/seanbreckenridge/dotfiles/blob/e11aea908ec4f2dd111143ebfe5d6a4eb07e268c/.config/zsh/functions/update#L11))) that I want to run periodically.

Putting all my jobs I want to run periodically in one [housekeeping](https://sean.fish/d/housekeeping?dark) script I run daily/weekly gives me the ability to monitor the output easily, but also allows me the flexibility of being able to schedule tasks to run at different rates. It also means that those scripts/commands can prompt me for input/confirmation, since this is run manually from a terminal, not in the background like cron.

This also means that all my 'cron-like' jobs are just bash scripts, and can be checked into version control easily.

I also have a [background loop script](https://github.com/seanbreckenridge/bgproc) that uses this to run tasks periodically, which I prefer to cron on my main machine. For examples of usage of `evry` there, you can look [here](https://github.com/seanbreckenridge/dotfiles/tree/master/.local/scripts/supervisor)
