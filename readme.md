# pmem

*Memory management for P-systems*

This library provides memory management facilities for P-systems.

---

### How to build
Building the library is relatively straightforward, requiring only a `cargo build` command to be
issued with Cargo's `build_script_build` doing the heavy lifting.

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
