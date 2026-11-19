//! The command line as a library entry point, so another tool (`repo`) can run it in-process
//! with the same settings, answers and exit codes as the `actit` binary.
use crate::args::{parse_args, parse_license, NameArgs};
use crate::contains;
use crate::error::Error;
use crate::forge_names::ForgeNames;
use crate::indexer::IndexerCatalog;
use crate::kag::{Ecosystems, KagVerifier};
use crate::license::{self, Check, Table};
use crate::name;
use crate::need::{parse_stopwords, Need, TermRules};
use crate::process::{KagProcess, UreqHttp};
use crate::report::{exit_code, render_licensed};
use crate::search::{find_with, Policy};
use crate::settings::Settings;
use actautoma::{gh::Gh, process::ProcessRunner};
use std::io::Write;

fn emit(out: &mut dyn Write, text: &str) -> Result<(), Error> {
    out.write_all(text.as_bytes())
        .map_err(|e| Error::Output(e.to_string()))
}

fn run_name(args: NameArgs, settings: &Settings, out: &mut dyn Write) -> Result<i32, Error> {
    let names = ForgeNames::new(Gh::new(ProcessRunner).program(&settings.gh));
    let repo = args.name.as_str();
    if args.contains {
        let listing = contains::list(repo, &names)?;
        emit(out, &contains::render(repo, &listing))?;
        return Ok(contains::exit_code(&listing));
    }
    let Some(owner) = args.owner else {
        let outcome = name::check_anywhere(repo, &names)?;
        emit(out, &name::render_anywhere(repo, &outcome))?;
        return Ok(name::exit_code(outcome.status));
    };
    let status = name::check(&owner, repo, &names)?;
    emit(out, &name::render(&owner, repo, status))?;
    Ok(name::exit_code(status))
}

fn term_rules(settings: &Settings) -> Result<TermRules, Error> {
    let mut rules = TermRules {
        min_length: settings.min_term_length,
        ..TermRules::default()
    };
    if let Some(path) = &settings.stopwords_file {
        let text = std::fs::read_to_string(path)
            .map_err(|e| Error::Usage(format!("--stopwords {path}: {e}")))?;
        rules.stopwords = parse_stopwords(&text);
    }
    Ok(rules)
}

fn search_need(text: &str, settings: &Settings) -> Result<Outcome, Error> {
    let need = Need::with_rules(text, &term_rules(settings)?)?;
    let catalog = IndexerCatalog::new(&settings.indexer, &UreqHttp);
    let ecosystems = Ecosystems::default().with(settings.ecosystems.clone());
    let verifier = KagVerifier::new(KagProcess::new(&settings.kag)).with_ecosystems(ecosystems);
    let policy = Policy {
        limit: settings.limit,
        min_confidence: settings.min_confidence,
        min_coverage: settings.min_coverage,
    };
    let report = find_with(&need, &catalog, &verifier, &policy)?;
    let licenses = license::checks(&report, &Table::builtin(), &settings.allow_license);
    Ok(Outcome { report, licenses })
}

fn run_need(text: &str, settings: &Settings, out: &mut dyn Write) -> Result<i32, Error> {
    let outcome = search_need(text, settings)?;
    emit(out, &render_licensed(&outcome.report, &outcome.licenses))?;
    Ok(exit_code(&outcome.report))
}

/// `--license <expr>`: 0 when the licence allows reuse under the policy, 1 when not.
fn run_license(expression: &str, settings: &Settings, out: &mut dyn Write) -> Result<i32, Error> {
    let verdict = license::check(Some(expression), &Table::builtin(), &settings.allow_license);
    emit(out, &format!("{}\n", license::render_line(&verdict)))?;
    Ok(i32::from(!verdict.allowed))
}

/// Run `actit <args>`: the answer goes to `out`, the return value is the exit code (0, 1; errors are `Err`).
pub fn dispatch(
    args: &[String],
    env: &dyn Fn(&str) -> Option<String>,
    out: &mut dyn Write,
) -> Result<i32, Error> {
    let (settings, rest) = Settings::parse(args, env)?;
    if let Some(expression) = parse_license(&rest)? {
        return run_license(&expression, &settings, out);
    }
    match parse_args(&rest)? {
        Some(parsed) => run_name(parsed, &settings, out),
        None => run_need(&rest.join(" "), &settings, out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: &[&str]) -> (Result<i32, Error>, Vec<u8>) {
        let args: Vec<String> = args.iter().map(ToString::to_string).collect();
        let mut out = Vec::new();
        let result = dispatch(&args, &|_| None, &mut out);
        (result, out)
    }

    fn usage(args: &[&str], needle: &str) {
        let (result, out) = run(args);
        match result {
            Err(Error::Usage(m)) => assert!(m.contains(needle), "{m}"),
            other => panic!("{other:?}"),
        }
        assert!(out.is_empty(), "nothing is printed on an error");
    }

    #[test]
    fn a_bad_setting_value_is_a_usage_error_naming_the_flag() {
        usage(&["--min-coverage", "3", "pdf"], "--min-coverage");
    }

    #[test]
    fn a_missing_stopwords_file_is_reported_by_name() {
        usage(
            &["--stopwords", "/no/such/words.txt", "pdf"],
            "/no/such/words.txt",
        );
    }

    #[test]
    fn an_empty_need_is_refused() {
        assert_eq!(run(&[]).0, Err(Error::EmptyNeed));
    }

    #[test]
    fn a_missing_gh_program_fails_the_name_check_and_names_it() {
        let (result, out) = run(&["--gh", "/no/such/gh", "--name", "x", "--owner", "o"]);
        assert!(matches!(result, Err(Error::Github(m)) if m.contains("not found")));
        assert!(out.is_empty());
    }
}

/// What a need search found, with the licence verdict of each finding (same order).
pub struct Outcome {
    pub report: crate::model::Report,
    pub licenses: Vec<Check>,
}

impl Outcome {
    /// The findings whose licence allows reuse.
    pub fn reusable(&self) -> Vec<&crate::model::Finding> {
        license::reusable(&self.report, &self.licenses)
    }
}

/// Run a need search (`args` are the CLI words: the need and any settings) without printing.
pub fn search(args: &[String], env: &dyn Fn(&str) -> Option<String>) -> Result<Outcome, Error> {
    let (settings, rest) = Settings::parse(args, env)?;
    search_need(&rest.join(" "), &settings)
}

#[cfg(test)]
mod search_tests {
    include!("search_cli_tests.rs");
}
