use crate::error::Error;
use crate::ports::RepoNames;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameStatus {
    Taken,
    Free,
}

fn valid(name: &str) -> bool {
    !matches!(name, "" | "." | "..")
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

type NameArgs = Option<(String, Option<String>)>;

/// `--name <repo> [--owner <o>]` selects a name check; anything else is a need.
pub fn parse_args(args: &[String]) -> Result<NameArgs, Error> {
    if args.first().map(String::as_str) != Some("--name") {
        return Ok(None);
    }
    let usage = || Error::Usage("actit --name <repo> [--owner <owner>]".into());
    match &args[1..] {
        [name] => Ok(Some((name.clone(), None))),
        [name, flag, owner] if flag == "--owner" => Ok(Some((name.clone(), Some(owner.clone())))),
        _ => Err(usage()),
    }
}

#[cfg(test)]
mod tests {
    include!("name_tests.rs");
}
