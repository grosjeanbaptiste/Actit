use crate::model::{Report, Verdict};
use std::fmt::Write;

/// 0: a documented tool exists. 1: none confirmed. 2: some candidate could not be read.
pub fn exit_code(report: &Report) -> i32 {
    match (report.failures.is_empty(), report.verdict) {
        (false, _) => 2,
        (true, Verdict::Exists) => 0,
        (true, _) => 1,
    }
}

pub fn render(report: &Report) -> String {
    let mut out = String::from(match report.verdict {
        Verdict::Exists => "EXISTS\n",
        Verdict::Unconfirmed => "UNCONFIRMED: candidates found, no documentation confirms them\n",
        Verdict::NoCandidate => "NO CANDIDATE: indexer holds no tool for these terms\n",
    });
    for f in &report.findings {
        let c = &f.candidate;
        let _ = writeln!(out, "- {} ({}) {}", c.id, c.registry, c.url); // String write never fails
        let _ = writeln!(
            out,
            "  {:.2} {} {}",
            f.evidence.confidence, f.evidence.location, f.evidence.snippet
        );
    }
    for (c, why) in &report.failures {
        let _ = writeln!(out, "! {} ({}): {why}", c.id, c.registry); // String write never fails
    }
    out
}

#[cfg(test)]
mod tests {
    include!("report_tests.rs");
}
