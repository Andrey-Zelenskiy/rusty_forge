# `rusty-forge`: General-purpose library of utility methods for numerical simulations

`rusty_forge` provides a collection of tools for setting up and running numerical simulations.
Designed for safety and speed, this module provides tools to efficiently set up a simulation project and initialize/save data in a safe and self-consistent manner.

## Features

* Simulation directory initialization;
* Protocols to archive, overwrite, ignore old data, or panic before taking any action;
* File managers to efficiently save data to files;
* Traits for structure initialization via Builder pattern;
* Trait for structured data output;

## Installation

Add to `Cargo.toml`:

```
[dependencies]
rusty_forge = "0.1.0"
```

## License

Licensed under BSD 3-Clause License.

