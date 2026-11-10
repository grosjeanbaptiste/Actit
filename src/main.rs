use actautoma::{gh::Gh, process::ProcessRunner};
use actit::args::{parse_args, NameArgs};
use actit::contains;
use actit::error::Error;
use actit::forge_names::ForgeNames;
use actit::indexer::IndexerCatalog;
use actit::kag::{Ecosystems, KagVerifier};
use actit::name;
use actit::need::{parse_stopwords, Need, TermRules};
use actit::process::{KagProcess, UreqHttp};
use actit::report::{exit_code, render};
use actit::search::{find_with, Policy};
use actit::settings::Settings;
use std::process::ExitCode;

fn run_name(args: NameArgs, settings: &Settings) -> Result<i32, Error> {
    let names = ForgeNames::new(Gh::new(ProcessRunner).program(&settings.gh));
    let repo = args.name.as_str();
    if args.contains {
        let listing = contains::list(repo, &names)?;
        print!("{}", contains::render(repo, &listing));
        return Ok(contains::exit_code(&listing));
    }
    let Some(owner) = args.owner else {
        let outcome = name::check_anywhere(repo, &names)?;
        print!("{}", name::render_anywhere(repo, &outcome));
        return Ok(name::exit_code(outcome.status));
    };
    let status = name::check(&owner, repo, &names)?;
    print!("{}", name::render(&owner, repo, status));
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

fn run(text: &str, settings: &Settings) -> Result<i32, Error> {
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
    print!("{}", render(&report));
    Ok(exit_code(&report))
}

fn dispatch(args: &[String]) -> Result<i32, Error> {
    let (settings, rest) = Settings::parse(args, &|key| std::env::var(key).ok())?;
    match parse_args(&rest)? {
        Some(parsed) => run_name(parsed, &settings),
        None => run(&rest.join(" "), &settings),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = dispatch(&args);
    match result {
        Ok(code) => ExitCode::from(code as u8), // codes are 0..=2
        Err(error) => {
            eprintln!("actit: {error}");
            ExitCode::from(2)
        }
    }
}
