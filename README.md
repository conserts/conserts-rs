<!--
SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE

SPDX-License-Identifier: MIT
-->

# conserts-rs

A library and set of command-line tools for working with ConSert models.

The goal of this library is to allow both safety engineers as well as embedded software developers to work with ConSert models.
These models come in the form of XML files, generated e.g. via the [safeTbox](https://safetbox.de/) software.
Usage of these models includes both runtime safety evaluation of single systems (where `conserts-rs` helps in auto-generating code) as well as the safety evaluation of the collaboration of several systems (where `conserts-rs` checks the composability).

## Disclaimer

This is the public read-only mirror of an internal conserts-rs repository. Pull requests will not be merged but the changes might be added internally and published with a future commit. Both repositories are synchronized from time to time.

The user DOES NOT get ANY WARRANTIES when using this tool. This software is released under the MIT License. By using this software, the user implicitly agrees to the licensing terms.

If you decide to use conserts-rs in your research please cite our papers.

## Functionality

`conserts-rs` supports the following workflows:

* [Parsing](#conserts-parse) (and validating) these models.
* [Plotting](#conserts-plot) the models, to see the graphical representation of the ConSert.
* [Compiling](#conserts-compile) Rust crates that contain logic for evaluating ConSerts at runtime. This crate is:
  * Usable for either
    * embedded targets that require `#![no_std]`,
    * [ROS](https://www.ros.org/) (Robot Operating System), as it brings ready to use ROS nodes along, or
    * any other system in which a binary generated from Rust can be deployed.
  * Unit-safe (with the help of [uom](https://crates.io/crates/uom)).
  * Usable through C/C++ bindings (with the help of [cbindgen](https://crates.io/crates/cbindgen)).
  * Automatically documented, by making connections from the ConSert model to the generated code.
* Validating the [composition](#conserts-compose) of several ConSert models.

## Setup

You can install `conserts-rs` as follows:

### Local Source

```sh
git clone git@github.com:conserts/conserts-rs.git
cargo install --path conserts
```

### Git

```sh
cargo install --git git@github.com:conserts/conserts-rs.git
```

## Usage

If you have installed Conserts as a binary or from source, you can run

```sh
conserts [command]
```

### `conserts parse`

You can parse a ConSert and print its internal representation to the console by calling:

```sh
conserts parse -i ./models/DEIS_DemoFollowerTruckSystem.model
```

### `conserts plot`

You can visualize a ConSert by generating a [`dot` language](https://graphviz.org/doc/info/lang.html)-compatible output:

```sh
conserts plot -i ./models/DEIS_DemoFollowerTruckSystem.model
```

By piping the standard output to a file, this can be persisted and passed to any visualization tool that is compatible with `dot`.

### `conserts compile`

Using `conserts compile`, you can generate a Rust crate that can be used to evaluate your ConSert at runtime.

```sh
conserts compile -i ./models/DEIS_DemoFollowerTruckSystem.model
```

This is going to create a crate in `target/consert_deis_demofollowertrucksystem` that contains a library including your ConSert's guarantees, demands, as well as runtime properties and services. Furthermore, it includes a monitor implementation.

### `conserts compose`

You can check if multiple ConSerts can be composed by calling:

```sh
conserts compose -i models/DEIS_DemoLeaderTruckSystem.model -i models/DEIS_DemoFollowerTruckSystem.model
```

If it succeeeds, a respective message it printed. If not, like in the following example, an error is printed:

```sh
conserts compose -i models/DEIS_DemoLeaderTruckSystemIncompatible.model -i models/DEIS_DemoFollowerTruckSystem.model`
```

## License

Licensed under MIT license.

## Science Behind

ConSerts have originally been developed in

```bibtex
@article{schneider:2014:conditional,
  title={Conditional Safety Certification for Open Adaptive Systems},
  author={Schneider, Daniel},
  year={2014},
  publisher={Fraunhofer IRB Verlag}
}
```
