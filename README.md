# breathe

[![build-badge](https://github.com/pizzamig/breathe/workflows/Rust/badge.svg)](https://github.com/pizzamig/breathe/actions)
[![Dependency Status](https://deps.rs/repo/github/pizzamig/breathe/status.svg)](https://deps.rs/repo/github/pizzamig/breathe)
[![GitHub latest commit](https://badgen.net/github/last-commit/pizzamig/breathe)](https://github.com/pizzamig/breathe/commit/)

`breathe` is a command line utility that contains breathing exercises

## Installation

Installation is optional, because we can use the dockerized version, as explained later.

However, if you want to install it and run it fully on your own, you'll need the Rust environment, because `breathe` is written in Rust.

To build and install `breate`, you have to:
```console
$ git clone git@github.com:pizzamig/breathe.git
$ cd breathe
$ cargo install --path .
$ cp resources/tests/config.toml $HOME/.config/breathe.toml
```

`breathe` will be installed in the `$HOME/.cargo/bin` folder.
Optionally, you can customize the configuration file `$HOME/.config.breathe.toml`.

## Dockerized version

A dockerized version of `breathe` is available on Dockerhub.

You can setup an alias in your shell, for convenience:
```
$ alias breathe="docker run -it --rm --net=host pizzamig/breathe:0.2.0"
$ breathe -l
```

## Configuration

The configuration file uses the TOML format and specifies the breathing patterns:
```toml
[patterns]

[patterns.relax]
description = "This breathing exercise is a natural tranquilizer for the nervous system. The 8 configuration pattern is not suggested for beginners."
breath_in = 4
breath_out = 8
hold_in = 7
counter_type = "Iteration"
duration = 8
```

Mandatory fields are:
* `description` : a string with a description of the pattern
* `breath_in`  : length (in seconds) of the inhale phase
* `breath_out` : length (in seconds) of the exhale phase

Other fields:
* `hold_in` : length (in seconds) of the break inhale and exhale [default value: 0]
* `hold_out` : length (in seconds) of the break after exhale [default value: 0]

A pattern is usually repeated multiple times, forming a session.
A session can be time based, i.e. 5 minutes, or iteration based,  i.e. repeat 8 times.

A time based session can be configured in this way (duration in seconds):
```toml
counter_type="Time"
duration = 300
```

An iteration based session can be configured in this way:
```toml
counter_type="Iteration"
duration = 8
```
Where `duration` is the number of iterations in a session.
