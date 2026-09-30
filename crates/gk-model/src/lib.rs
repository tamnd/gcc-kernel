//! The gcc-kernel model: what a cell is made of, and how the committed pin files describe it.
//!
//! Every coordinate of a cell is a name in a committed file (spec 02.1). This crate reads those files, checks that they agree with each other, and computes the identity of a cell from what its names resolve to. It runs nothing and downloads nothing.

pub mod cell;
pub mod eras;
pub mod hosts;
pub mod kernels;
pub mod ladder;
pub mod platforms;
pub mod repo;
pub mod sets;
pub mod toolchains;
pub mod version;

pub use ladder::{BadRung, Rung, Verdict};
pub use version::Version;

/// A value that changes with the kernel version, written in the pin files as a list of `{ from, value }` steps.
///
/// The value for a kernel is the one of the last step whose `from` is at or below it. A kernel older than the first step has none.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Step {
    /// The first kernel the value applies to.
    pub from: Version,
    /// The value.
    pub value: String,
}

/// The value of a list of steps for a kernel.
#[must_use]
pub fn step_for<'a>(steps: &'a [Step], kernel: &Version) -> Option<&'a str> {
    steps
        .iter()
        .filter(|s| s.from <= *kernel)
        .max_by(|a, b| a.from.cmp(&b.from))
        .map(|s| s.value.as_str())
}

/// Whether a string is a lower case hex SHA-256.
#[must_use]
pub fn is_sha256(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
