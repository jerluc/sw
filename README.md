# `sw` (stopwatch)

`sw` is a simple command-line program inspired by [GNU `time`](https://www.gnu.org/software/time/)
that times the execution of a shell command. Unlike GNU `time` however, `sw` also automatically
records historical timing statistics so you can see things like minimum, maximum, mean, and standard
deviation of historical runs of the same shell command over time.

## Installation

To get started, install `sw` from source:

```shell
git clone https://github.com/jerluc/sw.git && cd sw/ && cargo install --path .
```

## Usage

To use `sw`, simply use it like the `time` command:

```
sw <COMMAND> [COMMAND_ARGS...]
```

For example, the first time you run, you'll see the timing for the command:

```shell
sw sleep 1
# Took 1.002365579s
```

Then if you run the command again, you'll see the timing along with historical statistics:

```shell
sw sleep 1
# Took 1.001665857s
# Timing statistics for command [sleep 1]
#                Total: 2
#       Duration (min): 1.001665857s
#       Duration (max): 1.002365579s
#      Duration (mean): 1.002015718s
#   Duration (std dev): 349.861µs
```

## Internals

### Historical timings

Historical timings are grouped by command and arguments, and are recorded in a JSON file under the
user's local data directory:

- For Linux, this is `$XDG_DATA_HOME/sw/history.json` or `$HOME/.local/share/sw/history.json`
- For macOS, this is `$HOME/Library/Application Support/sw/history.json`
- For Windows, this is `{FOLDERID_LocalAppData}\sw\history.json`

This history file is automatically created on first run, so if you want to reset the history, simply
delete the file!

## Motivations

I basically had two motivations in creating this software:

1. I often rerun the same command multiple times to execute things like ETL jobs and other data
   pipelines. `sw` can be used to track how long these runs take on average compared to the current
   run, so that I can quickly identify when something is faster or slower than usual.
2. I wanted an excuse to practice some more Rust :)

## Contributing

When contributing to this repository, please follow the steps below:

1. Fork the repository
2. Submit your patch in one commit, or a series of well-defined commits
3. Submit your pull request and make sure you reference the issue you are addressing

## License

See [LICENSE](LICENSE)
