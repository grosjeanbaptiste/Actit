use crate::contains::Listing;
use crate::error::Error;
use crate::ports::RepoNames;
use actautoma::forge::Forge;

/// Asks the hosting platform through actautoma's `Forge` port (`gh` for GitHub). It sees the
/// owner's private repositories too.
pub struct ForgeNames<F: Forge> {
    forge: F,
}

impl<F: Forge> ForgeNames<F> {
    pub fn new(forge: F) -> Self {
        Self { forge }
    }
}

fn github(e: actautoma::forge::Error) -> Error {
    Error::Github(e.to_string())
}

impl<F: Forge> RepoNames for ForgeNames<F> {
    fn owner(&self) -> Result<String, Error> {
        self.forge.user().map_err(github)
    }

    fn exists(&self, owner: &str, name: &str) -> Result<bool, Error> {
        self.forge.repo_exists(owner, name).map_err(github)
    }

    fn search(&self, name: &str) -> Result<Vec<String>, Error> {
        let found = self.forge.search_repos(name).map_err(github)?;
        Ok(found
            .names
            .into_iter()
            .filter(|full| {
                full.split_once('/')
                    .is_some_and(|(_, n)| n.eq_ignore_ascii_case(name))
            })
            .collect())
    }

    fn contains(&self, text: &str) -> Result<Listing, Error> {
        let found = self.forge.search_repos(text).map_err(github)?;
        let lower = text.to_ascii_lowercase();
        let names = found
            .names
            .into_iter()
            .filter(|full| {
                let repo = full.split_once('/').map_or("", |(_, n)| n);
                repo.to_ascii_lowercase().contains(&lower)
            })
            .collect();
        Ok(Listing {
            total: found.total,
            names,
        })
    }
}

#[cfg(test)]
mod tests {
    include!("forge_names_tests.rs");
}
