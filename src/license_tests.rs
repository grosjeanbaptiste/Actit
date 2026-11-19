use super::*;
use crate::model::{Candidate, Evidence, Verdict};

fn t() -> Table {
    Table::builtin()
}

fn c(expr: &str) -> Check {
    check(Some(expr), &t(), &default_allow())
}

#[test]
fn class_names_round_trip_and_unknown_is_not_a_policy_word() {
    for class in [
        Class::PublicDomain,
        Class::Permissive,
        Class::WeakCopyleft,
        Class::StrongCopyleft,
        Class::NetworkCopyleft,
        Class::Restricted,
        Class::Proprietary,
        Class::Unknown,
    ] {
        assert_eq!(Class::parse(class.wire()), Some(class));
    }
    assert_eq!(Class::parse("permissive"), Some(Class::Permissive));
    assert_eq!(Class::parse("nonsense"), None);
}

#[test]
fn allow_list_parses_and_refuses_unknown_names() {
    assert_eq!(
        parse_allow("permissive, weak-copyleft"),
        Ok(vec![Class::Permissive, Class::WeakCopyleft])
    );
    assert_eq!(default_allow(), [Class::PublicDomain, Class::Permissive]);
    for bad in ["", "permissive,,mit", "mit", "unknown", "permissive,unknown"] {
        assert!(matches!(parse_allow(bad), Err(Error::License(_))), "{bad:?}");
    }
}

#[test]
fn a_table_file_has_one_id_and_class_per_line() {
    let table = Table::parse("# comment\n\nMIT|permissive\nfoo-1.0|restricted\n").unwrap();
    assert_eq!(table.class_of("foo-1.0"), Some(Class::Restricted));
    assert_eq!(table.class_of("FOO-1.0"), Some(Class::Restricted));
    for bad in ["MIT", "MIT|bogus", "MIT|permissive|x", "|permissive", "MIT|unknown"] {
        let got = Table::parse(&format!("# c\n{bad}\n"));
        assert!(matches!(got, Err(Error::License(ref m)) if m.contains("line 2")), "{bad:?}: {got:?}");
    }
    let dup = Table::parse("MIT|permissive\nmit|restricted\n");
    assert!(matches!(dup, Err(Error::License(ref m)) if m.contains("duplicate")), "{dup:?}");
}

#[test]
fn the_builtin_table_classifies_the_common_licences() {
    for (id, class) in [
        ("MIT", Class::Permissive),
        ("Apache-2.0", Class::Permissive),
        ("BSD-3-Clause", Class::Permissive),
        ("ISC", Class::Permissive),
        ("CC0-1.0", Class::PublicDomain),
        ("Unlicense", Class::PublicDomain),
        ("MPL-2.0", Class::WeakCopyleft),
        ("LGPL-3.0-or-later", Class::WeakCopyleft),
        ("GPL-3.0-only", Class::StrongCopyleft),
        ("GPL-2.0-or-later", Class::StrongCopyleft),
        ("AGPL-3.0-only", Class::NetworkCopyleft),
        ("SSPL-1.0", Class::NetworkCopyleft),
        ("BUSL-1.1", Class::Restricted),
        ("CC-BY-NC-4.0", Class::Restricted),
        ("UNLICENSED", Class::Proprietary),
    ] {
        assert_eq!(t().class_of(id), Some(class), "{id}");
    }
    assert_eq!(t().class_of("mit"), Some(Class::Permissive));
    assert_eq!(t().class_of("Not-A-Licence"), None);
}

#[test]
fn a_single_id_is_classified_and_checked_against_the_policy() {
    let mit = c("MIT");
    assert_eq!((mit.class, mit.allowed, mit.note), (Class::Permissive, true, None));
    let gpl = c("GPL-3.0-only");
    assert_eq!((gpl.class, gpl.allowed), (Class::StrongCopyleft, false));
    assert!(c("GPL-2.0+").class == Class::StrongCopyleft, "`+` means or-later");
}

#[test]
fn or_takes_the_least_restrictive_alternative_and_and_the_most() {
    assert_eq!(c("MIT OR GPL-3.0-only").class, Class::Permissive);
    assert_eq!(c("MIT OR Apache-2.0").class, Class::Permissive);
    assert_eq!(c("MIT AND GPL-3.0-only").class, Class::StrongCopyleft);
    assert_eq!(c("mit or apache-2.0").class, Class::Permissive);
    assert!(!c("MIT AND GPL-3.0-only").allowed);
}

#[test]
fn and_binds_tighter_than_or_and_parentheses_override() {
    // MIT OR (GPL AND AGPL) -> MIT
    assert_eq!(c("MIT OR GPL-3.0-only AND AGPL-3.0-only").class, Class::Permissive);
    // (MIT OR GPL) AND AGPL -> AGPL
    assert_eq!(c("(MIT OR GPL-3.0-only) AND AGPL-3.0-only").class, Class::NetworkCopyleft);
    assert_eq!(c("((MIT))").class, Class::Permissive);
}

#[test]
fn with_exception_keeps_the_class_of_the_licence() {
    assert_eq!(c("GPL-2.0-only WITH Classpath-exception-2.0").class, Class::StrongCopyleft);
}

#[test]
fn an_unknown_id_is_unknown_never_allowed_and_named() {
    let got = c("Frobnicate-1.0");
    assert_eq!((got.class, got.allowed), (Class::Unknown, false));
    assert!(got.note.as_deref().unwrap().contains("Frobnicate-1.0"), "{got:?}");
    // an OR with one known alternative is usable through that alternative
    assert_eq!(c("Frobnicate-1.0 OR MIT").class, Class::Permissive);
    // an AND cannot be vouched for with an unknown part
    assert_eq!(c("Frobnicate-1.0 AND MIT").class, Class::Unknown);
}

#[test]
fn missing_empty_or_malformed_expressions_are_unknown_with_a_reason() {
    for (expr, why) in [
        (None, "no licence declared"),
        (Some("  "), "no licence declared"),
        (Some("(MIT"), "parenthes"),
        (Some("MIT OR"), "expected"),
        (Some("MIT Apache-2.0"), "expected"),
        (Some("SEE LICENSE IN LICENSE.md"), "expected"),
        (Some("MIT )"), "unexpected"),
    ] {
        let got = check(expr, &t(), &default_allow());
        assert_eq!((got.class, got.allowed), (Class::Unknown, false), "{expr:?}");
        assert!(got.note.as_deref().unwrap_or("").contains(why), "{expr:?}: {got:?}");
    }
}

#[test]
fn the_policy_decides_and_unknown_is_never_allowed() {
    let weak = [Class::Permissive, Class::WeakCopyleft];
    assert!(check(Some("MPL-2.0"), &t(), &weak).allowed);
    assert!(!check(Some("GPL-3.0-only"), &t(), &weak).allowed);
    let all = [Class::PublicDomain, Class::Permissive, Class::WeakCopyleft, Class::StrongCopyleft,
        Class::NetworkCopyleft, Class::Restricted, Class::Proprietary, Class::Unknown];
    assert!(!check(Some("Frobnicate-1.0"), &t(), &all).allowed);
    assert!(!check(None, &t(), &all).allowed);
}

fn finding(id: &str, license: Option<&str>) -> Finding {
    Finding {
        candidate: Candidate {
            id: id.into(),
            registry: "crates_io".into(),
            url: format!("https://crates.io/crates/{id}"),
            docs_url: None,
            description: String::new(),
            license: license.map(str::to_owned),
        },
        term_hits: 1,
        evidence: Evidence { location: "l".into(), snippet: "s".into(), confidence: 0.9 },
    }
}

#[test]
fn checks_follow_the_findings_and_reusable_keeps_the_allowed_ones() {
    let report = Report {
        verdict: Verdict::Exists,
        findings: vec![finding("a", Some("MIT")), finding("b", Some("GPL-3.0-only")), finding("c", None)],
        failures: vec![],
    };
    let cs = checks(&report, &t(), &default_allow());
    assert_eq!(cs.iter().map(|x| x.allowed).collect::<Vec<_>>(), [true, false, false]);
    let kept: Vec<_> = reusable(&report, &cs).iter().map(|f| f.candidate.id.clone()).collect();
    assert_eq!(kept, ["a"]);
}

#[test]
fn a_line_names_the_expression_class_and_verdict() {
    assert_eq!(render_line(&c("MIT OR Apache-2.0")), "licence MIT OR Apache-2.0: permissive (reusable)");
    assert_eq!(render_line(&c("GPL-3.0-only")), "licence GPL-3.0-only: strong-copyleft (not reusable)");
    let none = check(None, &t(), &default_allow());
    assert_eq!(render_line(&none), "licence none declared: unknown (not reusable), no licence declared");
}
