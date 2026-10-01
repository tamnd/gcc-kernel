//! `gk-tap`: the KUnit results in a boot log (spec 07.5).
//!
//! A cell boots for KUnit once per run, and [`tally`] lines the runs up suite by suite. A suite is graded for a kernel when the era GCC's cell passed it in every run, and a cell fails L7 when any run of it did not pass a graded suite (spec 07.5).
//!
//! KUnit prints KTAP on the console as its suites run at boot. A suite's result is an `ok` or `not ok` line with no indent, and its cases are the same lines indented once, after a `# Subtest:` line that names the suite. Parameterised cases nest one level deeper and are counted in their case's line, so only the first two levels are read. The kernel's time stamps come off first, and lines from anything else are skipped, since printk can put them between KUnit's.

use crate::boot::strip_timestamp;
use serde::{Deserialize, Serialize};

/// How a suite or a case ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// `ok`.
    Pass,
    /// `not ok`.
    Fail,
    /// `ok` with a `# SKIP` directive.
    Skip,
}

/// One case of a suite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Case {
    /// Its name.
    pub name: String,
    /// How it ended.
    pub status: Status,
}

/// One suite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suite {
    /// Its name.
    pub name: String,
    /// How it ended, or `None` when the log stops before its result line.
    pub status: Option<Status>,
    /// Its cases, in order.
    pub cases: Vec<Case>,
}

/// A result line, split into the status and the name: `ok 3 name`, `not ok 3 name`, or either with `# SKIP why` after.
fn result_line(text: &str) -> Option<(Status, &str)> {
    let (ok, rest) = if let Some(rest) = text.strip_prefix("not ok ") {
        (false, rest)
    } else {
        (true, text.strip_prefix("ok ")?)
    };
    let (number, rest) = rest.split_once(' ').unwrap_or((rest, ""));
    if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let (name, directive) = rest.split_once(" # ").unwrap_or((rest, ""));
    let status = if !ok {
        Status::Fail
    } else if directive
        .trim_start()
        .to_ascii_uppercase()
        .starts_with("SKIP")
    {
        Status::Skip
    } else {
        Status::Pass
    };
    Some((status, name.trim()))
}

/// Every suite in a boot log, in the order they ran.
#[must_use]
pub fn suites(log: &str) -> Vec<Suite> {
    let mut out: Vec<Suite> = Vec::new();
    let mut open: Option<Suite> = None;
    for raw in log.lines() {
        let line = strip_timestamp(raw.trim_end_matches('\r'));
        let indent = line.len() - line.trim_start_matches(' ').len();
        let text = line.trim_start_matches(' ');
        if indent == 4
            && let Some(name) = text.strip_prefix("# Subtest: ")
        {
            if let Some(s) = open.take() {
                out.push(s);
            }
            open = Some(Suite {
                name: name.trim().to_owned(),
                status: None,
                cases: Vec::new(),
            });
            continue;
        }
        let Some((status, name)) = result_line(text) else {
            continue;
        };
        match indent {
            0 => {
                let mut s = match open.take() {
                    Some(s) if s.name == name => s,
                    other => {
                        if let Some(s) = other {
                            out.push(s);
                        }
                        Suite {
                            name: name.to_owned(),
                            status: None,
                            cases: Vec::new(),
                        }
                    }
                };
                s.status = Some(status);
                out.push(s);
            }
            4 => {
                if let Some(s) = open.as_mut() {
                    s.cases.push(Case {
                        name: name.to_owned(),
                        status,
                    });
                }
            }
            _ => {}
        }
    }
    if let Some(s) = open {
        out.push(s);
    }
    out
}

/// One suite over every KUnit boot of a cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tally {
    /// The suite's name.
    pub name: String,
    /// How it ended in each run, `None` where the run never got to its result.
    pub runs: Vec<Option<Status>>,
}

impl Tally {
    /// Whether every one of `runs` boots passed the suite.
    #[must_use]
    pub fn passed_all(&self, runs: usize) -> bool {
        self.runs.len() == runs && self.runs.iter().all(|r| *r == Some(Status::Pass))
    }
}

/// The suites of several boots, by name in the order they first ran.
#[must_use]
pub fn tally(boots: &[Vec<Suite>]) -> Vec<Tally> {
    let mut out: Vec<Tally> = Vec::new();
    for (n, suites) in boots.iter().enumerate() {
        for s in suites {
            let at = if let Some(at) = out.iter().position(|t| t.name == s.name) {
                at
            } else {
                out.push(Tally {
                    name: s.name.clone(),
                    runs: Vec::new(),
                });
                out.len() - 1
            };
            out[at].runs.resize(n, None);
            out[at].runs.push(s.status);
        }
    }
    for t in &mut out {
        t.runs.resize(boots.len(), None);
    }
    out
}

/// The suites the reference passed in every one of its `runs` boots, which are the ones graded.
#[must_use]
pub fn graded(reference: &[Tally], runs: usize) -> Vec<&str> {
    reference
        .iter()
        .filter(|t| t.passed_all(runs))
        .map(|t| t.name.as_str())
        .collect()
}

/// The graded suites one boot did not pass, missing ones included.
#[must_use]
pub fn failed<'a>(boot: &[Suite], graded: &[&'a str]) -> Vec<&'a str> {
    graded
        .iter()
        .filter(|g| {
            !boot
                .iter()
                .any(|s| s.name == **g && s.status == Some(Status::Pass))
        })
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOG: &str = "\
[    3.451604] KTAP version 1
[    3.451700] 1..2
[    3.459989]     KTAP version 1
[    3.460000]     # Subtest: list-kernel-test
[    3.460100]     # module: list_test
[    3.460200]     1..3
[    3.460300]     ok 1 list_test_list_init
[    3.460400] random: crng init done
[    3.460500]     not ok 2 list_test_list_add
[    3.460600]     ok 3 list_test_list_del # SKIP not on this arch
[    3.460700] # list-kernel-test: pass:1 fail:1 skip:1 total:3
[    3.460800] not ok 1 list-kernel-test
[  180.475231]     # Subtest: amdv1_iommu_test
[  180.475300]         KTAP version 1
[  180.475400]         # Subtest: test_init
[  180.475500]         not ok 1 amdv1_cfg_0
[  180.475600]         ok 2 amdv1_cfg_1
[  180.616749]     not ok 1 test_init
[  180.616800]     ok 2 test_bitops
[  180.601899] ok 2 amdv1_iommu_test
";

    #[test]
    fn suites_and_cases_are_read() {
        let s = suites(LOG);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].name, "list-kernel-test");
        assert_eq!(s[0].status, Some(Status::Fail));
        let cases: Vec<_> = s[0].cases.iter().map(|c| c.status).collect();
        assert_eq!(cases, [Status::Pass, Status::Fail, Status::Skip]);
        assert_eq!(s[1].name, "amdv1_iommu_test");
        assert_eq!(s[1].status, Some(Status::Pass));
        assert_eq!(s[1].cases.len(), 2);
        assert_eq!(s[1].cases[0].name, "test_init");
    }

    #[test]
    fn a_log_cut_short_leaves_the_suite_open() {
        let s = suites(
            "    # Subtest: rtc_lib_test_cases\n    1..2\n    ok 1 rtc_time64_to_tm_test_date_range\n",
        );
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].status, None);
        assert_eq!(s[0].cases.len(), 1);
    }

    #[test]
    fn result_lines_need_a_number() {
        assert_eq!(result_line("ok 12 a_b"), Some((Status::Pass, "a_b")));
        assert_eq!(
            result_line("ok 1 x # SKIP no hardware"),
            Some((Status::Skip, "x"))
        );
        assert_eq!(result_line("not ok 3 y"), Some((Status::Fail, "y")));
        assert_eq!(result_line("ok then"), None);
        assert_eq!(result_line("okay 1 z"), None);
    }

    #[test]
    fn runs_are_tallied_and_graded() {
        let suite = |name: &str, status| Suite {
            name: name.into(),
            status,
            cases: Vec::new(),
        };
        let one = vec![
            suite("a", Some(Status::Pass)),
            suite("b", Some(Status::Pass)),
        ];
        let two = vec![
            suite("a", Some(Status::Pass)),
            suite("b", Some(Status::Fail)),
        ];
        let three = vec![
            suite("a", Some(Status::Pass)),
            suite("c", Some(Status::Pass)),
        ];
        let t = tally(&[one.clone(), two.clone(), three.clone()]);
        assert_eq!(t.len(), 3);
        assert_eq!(t[1].runs, [Some(Status::Pass), Some(Status::Fail), None]);
        assert_eq!(t[2].runs, [None, None, Some(Status::Pass)]);
        let g = graded(&t, 3);
        assert_eq!(g, ["a"]);
        assert!(failed(&two, &g).is_empty());
        assert_eq!(failed(&[suite("b", Some(Status::Pass))], &g), ["a"]);
        assert!(graded(&t, 4).is_empty());
    }
}
