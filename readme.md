# pmem

*Memory management for P-systems*

This library provides memory management facilities for P-systems.

---


### How to build

In order to build a standalone binary for PMEM, use the build script provided in the 'bin'
folder: `bin/psys.sh`. This script allows you to select one of three build targets (x86, RISC-V, and ARM),
and in addition to building the project, you can also run tests and benchmarks with it.

Some arguments can be passed to the build script as well:
* -h > Displays a help message.
* -b > Build the project. Takes an optional string argument to specify build target.
* -t > Runs project tests and benchmarks. Takes an optional string argument to specify target.
* -c > Runs the specified command.
* -d > Builds HTML documentation. (Coming soon).

Examples:
* `bin/psys.sh -h`
* `bin/psys.sh -b x86_64`
* `bin/psys.sh -t rv64gc`
* `bin/psys.sh -c build`
* `bin/psys.sh -d arm`


### How to use
The library can be used in any of your projects by including it in your package manifest:
```toml
[package]
name = "<name>"
version = "0.1.0"
authors = ["<NAME> <EMAIL>"]
edition = "2018"

[dependencies]
pmem = { git = "https://github.com/mnimi/pmem", branch = "trunk" }
```

---

### Licensing
Any usage of this project is required to conform to the modified BSD license.

### Contact
If you have any questions or want to contribute, [Tweet at me](https://twitter.com/yarotk).
