use actit::error::Error;
use actit::github::GhNames;
use actit::indexer::IndexerCatalog;
use actit::kag::KagVerifier;
use actit::name;
use actit::need::Need;
use actit::ports::RepoNames;
use actit::process::{KagProcess, UreqHttp};
use actit::report::{exit_code, render};
use actit::search::find;
use std::process::ExitCode;

const INDEXER: &str = "https://indexer.acteble.workers.dev";
const VERIFIED: usize = 5;

fn run_name(repo: &str, owner: Option<String>) -> Result<i32, Error> {
    let names = GhNames::new(KagProcess::new("gh"));
    let owner = match owner {
        Some(owner) => owner,
        None => names.owner()?,
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
    let result = match name::parse_args(&args) {
        Ok(Some((repo, owner))) => run_name(&repo, owner),
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
