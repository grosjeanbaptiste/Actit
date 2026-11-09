use actit::indexer::IndexerCatalog;
use actit::kag::KagVerifier;
use actit::need::Need;
use actit::process::{KagProcess, UreqHttp};
use actit::report::{exit_code, render};
use actit::search::find;
use std::process::ExitCode;

const INDEXER: &str = "https://indexer.acteble.workers.dev";
const VERIFIED: usize = 5;

fn run(text: &str) -> Result<i32, actit::error::Error> {
    let need = Need::new(text)?;
    let catalog = IndexerCatalog::new(INDEXER, &UreqHttp);
    let verifier = KagVerifier::new(KagProcess::new("kag"));
    let report = find(&need, &catalog, &verifier, VERIFIED)?;
    print!("{}", render(&report));
    Ok(exit_code(&report))
}

fn main() -> ExitCode {
    let text = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    match run(&text) {
        Ok(code) => ExitCode::from(code as u8), // codes are 0..=2
        Err(error) => {
            eprintln!("actit: {error}");
            ExitCode::from(2)
        }
    }
}
