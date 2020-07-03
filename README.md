# evry

A tool to *manually* run commands -- periodically.

* Documentation
* Fix Help Message

evry 2 months -selenium && {
# evry succeeded, so the external command should be run
    python selenium.py || {
        # the python process exited with a non-zero exit code
        # we should rollback when the command was last run, so
        # we can re-try later
        evry rollback -selenium
    }
}

Note: durations are additive, multiple 5m will be added together
