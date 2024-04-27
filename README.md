<div align="center">
    <h1>shigan</h1>
    <b>Command-line Time Tracker written in Rust</b>
    <div>
    	<a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
    </div>
    <img src="./shigan.png" alt="logo" />
  <br />
</div>


## Installation

### Build from source

First, install `rustup` to get the `rust` compiler using `curl https://sh.rustup.rs -sSf | sh -s`. Then, 

```sh
$ git clone https://github.com/img9417/shigan
$ cd shigan
$ cargo build --release

$ ./target/release/shigan
```

### Homebrew

```shell
$ brew tap img9417/shigan
$ brew install shigan
```

To update, run 

```sh
$ brew update
$ brew upgrade shigan
```

## Usage
```text
Command line Time Tracker

Usage: shigan [COMMAND]

Commands:
  add     Add a task to the tracker
  delete  Deletes a task from the tracker
  start   Starts the tracker for <TASK>
  stop    Stops currently running tracker
  log     List accumulated time for the task or all (default="all")
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Add a task

You can add a task using `add -t <TASK> ` or `add --task <TASK>`

```sh
$ shigan add -t Programming
```

This data will be saved in `data.json` inside `.shigan` directory located in the root directory.
The directory and the file will be created automatically if not exist.

Tasks are NOT case sensitive as everything will be transformed to a lowercase.

### Delete a task

You can remove the task using `delete -t <TASK>` or `delete --task <TASK>`

```sh
$ shigan delete -t programming
```

If such task doesn't exist, Shigan will throw an error.

```sh
$ shigan delete --task prog
@@ Task 'prog' not found
```

### Start the tracker

You can start tracking for a specific task using `start -t <TASK>` or `start --task <TASK>`.

```sh
$ shigan start -t programming
@@ Task 'programming' starting
```

If a task doesn't exist, Shigan will throw an error.

```sh
$ shigan start -t prog
@@ Task 'prog' does not exist.
```

### Stop the tracker

You can stop the tracker simply by using `shigan stop`.

```sh
$ shigan stop
Stopped tracking for the task "programming"
```

This will stop the tracker and record the session in `~/.shigan/data.json`.

If there is no ongoing task, Shigan will throw an error.

```sh
$ shigan stop
@@ Error - there's no ongoing task.
```

### Log the time

You can use `log` to display all the tasks and their accumulated time.

```text
$ shigan log

+-------------+------------------+
| Tasks       | Time Accumulated |
+-------------+------------------+
| programming |      2h  55m     |
+-------------+------------------+
| reading     |      1h  17m     |
+-------------+------------------+
| english     |      0h  17m     |
+-------------+------------------+
```

You can specify the task to display its time only.

```text
$ shigan log -t programming

+-------------+------------------+
| Tasks       | Time Accumulated |
+-------------+------------------+
| programming |      2h  55m     |
+-------------+------------------+
```

## Crates used
- chrono (0.4.37)
- clap (4.5.4)
- dirs (5.0.1)
- prettytable-rs (0.10.0)
- serde_json (1.0.115)
