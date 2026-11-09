use crate::indexer::Http;
use crate::kag::Runner;
use std::process::Command;

/// Runs the real `kag` binary.
pub struct KagProcess {
    program: String,
}

impl KagProcess {
    pub fn new(program: &str) -> Self {
        Self {
            program: program.to_string(),
        }
    }
}

impl Runner for KagProcess {
    fn run(&self, args: &[String]) -> Result<String, String> {
        let out = Command::new(&self.program)
            .args(args)
            .output()
            .map_err(|e| format!("cannot run {}: {e}", self.program))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(format!(
                "{} exited with {}: {}",
                self.program,
                out.status,
                stderr.trim()
            ));
        }
        String::from_utf8(out.stdout)
            .map_err(|e| format!("{} printed non UTF-8: {e}", self.program))
    }
}

/// Real HTTP transport for indexer.
pub struct UreqHttp;

impl Http for UreqHttp {
    fn get(&self, url: &str) -> Result<String, String> {
        let response = ureq::get(url)
            .call()
            .map_err(|e| format!("GET {url}: {e}"))?;
        response
            .into_string()
            .map_err(|e| format!("GET {url}: unreadable body: {e}"))
    }
}

#[cfg(test)]
mod tests {
    include!("process_tests.rs");
}
