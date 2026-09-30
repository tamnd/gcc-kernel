//! `gk-cc`, the tracing compiler shim.
//!
//! kbuild calls `$(CC)` tens of thousands of times in a build: once per unit, once per `.S` file, and several hundred times more while Kconfig and the Makefiles ask the compiler what it can do. `gk cell` points all of those at `gk-cc`, which runs the real compiler with the arguments unchanged and appends one line of JSON to `compile.jsonl` for every call.
//!
//! This is rucc-kernel's `rk-cc` under another name, copied rather than shared (spec 10.4). The record has the same schema, so rucc-kernel's tools read a gcc-kernel build directory without change. The rucc trace and the bring-up delegation are left out, because nothing here ever runs anything but a real GCC.
//!
//! This library holds the parts a test can reach: the record, the reading of a command line, the configuration, and hashing. The binary in `main.rs` is the glue that runs the compiler.

pub mod args;
pub mod config;
pub mod digest;
pub mod record;
pub mod usage;
