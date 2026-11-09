use crate::error::Error;
use crate::ports::RepoNames;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameStatus {
    Taken,
    Free,
}

/// GitHub's rules: 1 to 100 of `[A-Za-z0-9._-]`, not `.`/`..`, not ending in `.git`.
pub(crate) fn valid(name: &str) -> bool {
    (1..=100).contains(&name.len())
        && !matches!(name, "." | "..")
        && !name.to_ascii_lowercase().ends_with(".git")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
}

/// Is `owner/name` already a GitHub repository?
pub fn check(owner: &str, name: &str, names: &impl RepoNames) -> Result<NameStatus, Error> {
    if !valid(name) {
        return Err(Error::InvalidName(name.to_string()));
    }
    Ok(if names.exists(owner, name)? {
        NameStatus::Taken
    } else {
        NameStatus::Free
    })
}

/// What a GitHub-wide check found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub status: NameStatus,
    pub matches: Vec<String>,
}

/// Is any repository, under any owner, called `name`? Your own repositories
/// (private ones included) are checked exactly; the rest comes from GitHub search.
pub fn check_anywhere(name: &str, names: &impl RepoNames) -> Result<Outcome, Error> {
    valid(name)
        .then_some(())
        .ok_or_else(|| Error::InvalidName(name.to_string()))?;
    let owner = names.owner()?;
    let mut matches = names.search(name)?;
    let own = format!("{owner}/{name}");
    if names.exists(&owner, name)? && !matches.iter().any(|m| m.eq_ignore_ascii_case(&own)) {
        matches.insert(0, own);
    }
    let status = if matches.is_empty() {
        NameStatus::Free
    } else {
        NameStatus::Taken
    };
    Ok(Outcome { status, matches })
}

const SHOWN: usize = 10;

pub fn render_anywhere(name: &str, outcome: &Outcome) -> String {
    if outcome.status == NameStatus::Free {
        return format!("FREE {name} (none among the first 100 best GitHub matches)\n");
    }
    let mut text = format!("TAKEN {name}\n");
    for m in outcome.matches.iter().take(SHOWN) {
        text.push_str(&format!("- {m}\n"));
    }
    if outcome.matches.len() > SHOWN {
        text.push_str(&format!("(+{} more)\n", outcome.matches.len() - SHOWN));
    }
    text
}

/// Same convention as a need: 0 found, 1 not found.
pub fn exit_code(status: NameStatus) -> i32 {
    i32::from(status == NameStatus::Free)
}

pub fn render(owner: &str, name: &str, status: NameStatus) -> String {
    let word = if status == NameStatus::Taken {
        "TAKEN"
    } else {
        "FREE"
    };
    format!("{word} {owner}/{name}\n")
}

#[cfg(test)]
mod tests {
    include!("name_tests.rs");
}
