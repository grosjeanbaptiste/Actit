use crate::error::Error;
use crate::kag::Runner;
use crate::ports::RepoNames;

/// Asks GitHub through `gh`, which sees the owner's private repositories too.
pub struct GhNames<R: Runner> {
    runner: R,
}

impl<R: Runner> GhNames<R> {
    pub fn new(runner: R) -> Self {
        Self { runner }
    }
}

impl<R: Runner> RepoNames for GhNames<R> {
    fn owner(&self) -> Result<String, Error> {
        let args = ["api", "user", "-q", ".login"].map(String::from);
        let out = self.runner.run(&args).map_err(Error::Github)?;
        match out.trim() {
            "" => Err(Error::Github("gh returned an empty login".into())),
            login => Ok(login.to_string()),
        }
    }

    fn exists(&self, owner: &str, name: &str) -> Result<bool, Error> {
        let args = ["api".to_string(), format!("repos/{owner}/{name}")];
        match self.runner.run(&args) {
            Ok(_) => Ok(true),
            Err(reason) if reason.contains("Not Found") => Ok(false),
            Err(reason) => Err(Error::Github(reason)),
        }
    }

    fn search(&self, name: &str) -> Result<Vec<String>, Error> {
        let query = format!("search/repositories?q={name}+in:name&per_page=100");
        let args = ["api", &query, "-q", ".items[].full_name"].map(String::from);
        let out = self.runner.run(&args).map_err(Error::Github)?;
        Ok(out
            .lines()
            .filter(|full| {
                full.split_once('/')
                    .is_some_and(|(_, n)| n.eq_ignore_ascii_case(name))
            })
            .map(String::from)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    include!("github_tests.rs");
}
