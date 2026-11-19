use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let env = |key: &str| std::env::var(key).ok();
    match actit::cli::dispatch(&args, &env, &mut std::io::stdout().lock()) {
        Ok(code) => ExitCode::from(code as u8), // codes are 0..=2
        Err(error) => {
            eprintln!("actit: {error}");
            ExitCode::from(2)
        }
    }
}
