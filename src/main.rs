use actit::args::{parse_args, NameArgs};
use actit::contains;
use actit::error::Error;
use actit::github::GhNames;
use actit::indexer::IndexerCatalog;
use actit::kag::KagVerifier;
use actit::name;
use actit::need::Need;
use actit::process::{KagProcess, UreqHttp};
use actit::report::{exit_code, render};
use actit::search::find;
use std::process::ExitCode;

const INDEXER: &str = "https://indexer.acteble.workers.dev";
const VERIFIED: usize = 5;

fn run_name(args: NameArgs) -> Result<i32, Error> {
    let names = GhNames::new(KagProcess::new("gh"));
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

fn run(text: &str) -> Result<i32, Error> {
    let need = Need::new(text)?;
    let catalog = IndexerCatalog::new(INDEXER, &UreqHttp);
    let verifier = KagVerifier::new(KagProcess::new("kag"));
    let report = find(&need, &catalog, &verifier, VERIFIED)?;
    print!("{}", render(&report));
    Ok(exit_code(&report))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match parse_args(&args) {
        Ok(Some(parsed)) => run_name(parsed),
        Ok(None) => run(&args.join(" ")),
        Err(error) => Err(error),
    };
    match result {
        Ok(code) => ExitCode::from(code as u8), // codes are 0..=2
        Err(error) => {
            eprintln!("actit: {error}");
            ExitCode::from(2)
        }
    }
}
