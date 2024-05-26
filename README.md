# ensan

![Crates.io Version](https://img.shields.io/crates/v/ensan)
![Crates.io License](https://img.shields.io/crates/l/ensan)


> A corrosive expression engine for a corroded language.

Extended (Dynamic) Evaluation Engine for `hcl-rs`.

This crate aims to provide an extended evaluation engine for `hcl-rs`.
It simplifies the usage of `hcl-rs` by re-implementing features in `hcldec` library.
It includes advanced DAG graph building and evaluation features, allowing evaluation of references in the current document without needing additional variables in the context, similar to Terraform's behavior.

## Features
 - Re-implementations of the HCL2 built-in functions from Terraform, Packer, and other HashiCorp tools.
 - Out-of-the-box support for evaluating references in the current document.
 - Simple API for evaluating entire documents with serde serialization support.

For usage, see the documentation for the [`engine`] module.


