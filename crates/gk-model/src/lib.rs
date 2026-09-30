//! The gcc-kernel model: the outcome ladder a cell climbs and the verdict it earns.
//!
//! The definitions follow `docs/spec/02-the-question.md`. Cells, pins and identity arrive in G0.

use std::fmt;
use std::str::FromStr;

/// A rung of the outcome ladder, from L0 fetched to L8 clean.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rung {
    /// The tree's SHA-256 matches the pin, and it unpacks.
    Fetched,
    /// The kernel's own compiler checks accept the GCC.
    Accepted,
    /// The configuration step finishes with every option the fragment asked for.
    Configured,
    /// Every selected translation unit compiles and assembles.
    Compiled,
    /// `vmlinux` links and the boot image is produced.
    Linked,
    /// The kernel reaches `gk-init` and prints the boot marker in time.
    Booted,
    /// The era's smoke suite passes and the machine powers off.
    Smoke,
    /// Every KUnit suite and self test the fragment builds passes.
    Tested,
    /// No objtool warning in the build log and no splat on the console.
    Clean,
}

impl Rung {
    /// Every rung, in ladder order.
    pub const ALL: [Rung; 9] = [
        Rung::Fetched,
        Rung::Accepted,
        Rung::Configured,
        Rung::Compiled,
        Rung::Linked,
        Rung::Booted,
        Rung::Smoke,
        Rung::Tested,
        Rung::Clean,
    ];

    /// The rung's number on the ladder, 0 to 8.
    #[must_use]
    pub fn level(self) -> u8 {
        self as u8
    }

    /// The rung's short name, as `cell.json` writes it.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Rung::Fetched => "fetched",
            Rung::Accepted => "accepted",
            Rung::Configured => "configured",
            Rung::Compiled => "compiled",
            Rung::Linked => "linked",
            Rung::Booted => "booted",
            Rung::Smoke => "smoke",
            Rung::Tested => "tested",
            Rung::Clean => "clean",
        }
    }
}

impl fmt::Display for Rung {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "L{}", self.level())
    }
}

/// The error for a rung that is not `L0` to `L8`.
#[derive(Debug, PartialEq, Eq)]
pub struct BadRung(pub String);

impl fmt::Display for BadRung {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a rung: {:?} (expected L0 to L8)", self.0)
    }
}

impl std::error::Error for BadRung {}

impl FromStr for Rung {
    type Err = BadRung;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix('L')
            .and_then(|n| n.parse::<usize>().ok())
            .and_then(|n| Rung::ALL.get(n).copied())
            .ok_or_else(|| BadRung(s.to_owned()))
    }
}

/// What the matrix shows for a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Verdict {
    /// Reached L6, and L7 and L8 where the kernel has them.
    Works,
    /// Reached L6 and failed L7 or L8.
    Runs,
    /// Reached L4 and failed L5 or L6.
    Builds,
    /// Stopped before L4.
    Fails,
    /// The cell cannot exist on this platform.
    NotApplicable,
}

impl Verdict {
    /// The verdict of a cell whose highest passed rung is `reached`, for a kernel whose ladder ends at `top`.
    ///
    /// `top` is L6 for a kernel without KUnit or a splat check, and L8 for one that has both. `reached` is `None` when not even L0 passed.
    #[must_use]
    pub fn of(reached: Option<Rung>, top: Rung) -> Verdict {
        match reached {
            Some(r) if r >= top.max(Rung::Smoke) => Verdict::Works,
            Some(r) if r >= Rung::Smoke => Verdict::Runs,
            Some(r) if r >= Rung::Linked => Verdict::Builds,
            _ => Verdict::Fails,
        }
    }

    /// The single character the heat maps use.
    #[must_use]
    pub fn letter(self) -> char {
        match self {
            Verdict::Works => 'W',
            Verdict::Runs => 'R',
            Verdict::Builds => 'B',
            Verdict::Fails => 'F',
            Verdict::NotApplicable => '·',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rungs_round_trip() {
        for r in Rung::ALL {
            assert_eq!(r.to_string().parse::<Rung>(), Ok(r));
        }
        assert!("L9".parse::<Rung>().is_err());
        assert!("3".parse::<Rung>().is_err());
    }

    #[test]
    fn verdicts_follow_the_ladder() {
        assert_eq!(Verdict::of(None, Rung::Clean), Verdict::Fails);
        assert_eq!(
            Verdict::of(Some(Rung::Compiled), Rung::Clean),
            Verdict::Fails
        );
        assert_eq!(
            Verdict::of(Some(Rung::Linked), Rung::Clean),
            Verdict::Builds
        );
        assert_eq!(
            Verdict::of(Some(Rung::Booted), Rung::Clean),
            Verdict::Builds
        );
        assert_eq!(Verdict::of(Some(Rung::Smoke), Rung::Clean), Verdict::Runs);
        assert_eq!(Verdict::of(Some(Rung::Clean), Rung::Clean), Verdict::Works);
    }

    #[test]
    fn museum_kernels_work_at_smoke() {
        assert_eq!(Verdict::of(Some(Rung::Smoke), Rung::Smoke), Verdict::Works);
        assert_eq!(
            Verdict::of(Some(Rung::Linked), Rung::Smoke),
            Verdict::Builds
        );
    }
}
