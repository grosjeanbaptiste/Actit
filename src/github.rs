use crate::contains::Listing;
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
        let args = search_args(name, ".items[].full_name");
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

    fn contains(&self, text: &str) -> Result<Listing, Error> {
        let args = search_args(text, ".total_count, .items[].full_name");
        let out = self.runner.run(&args).map_err(Error::Github)?;
        let mut lines = out.lines();
        let total = lines.next().and_then(|l| l.trim().parse().ok());
        let total = total.ok_or_else(|| Error::Github("unreadable total_count".into()))?;
        let lower = text.to_ascii_lowercase();
        let names = lines
            .filter(|full| {
                let repo = full.split_once('/').map_or("", |(_, n)| n);
                repo.to_ascii_lowercase().contains(&lower)
            })
            .map(String::from)
            .collect();
        Ok(Listing { total, names })
    }
}

fn search_args(text: &str, jq: &str) -> Vec<String> {
    let query = format!("search/repositories?q={text}+in:name&per_page=100");
    ["api", &query, "-q", jq].map(String::from).to_vec()
}

#[cfg(test)]
mod tests {
    include!("github_tests.rs");
}
