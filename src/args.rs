use crate::error::Error;

/// A parsed `--name` invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameArgs {
    pub name: String,
    pub owner: Option<String>,
    pub contains: bool,
}

const USAGE: &str = "actit --name <repo> [--owner <owner> | --contains]";

/// `--name <repo> [--owner <o> | --contains]` selects a name check; anything else is a need.
pub fn parse_args(args: &[String]) -> Result<Option<NameArgs>, Error> {
    if args.first().map(String::as_str) != Some("--name") {
        return Ok(None);
    }
    let usage = || Error::Usage(USAGE.into());
    let mut rest = args[1..].iter();
    let name = rest
        .next()
        .filter(|n| !n.starts_with("--"))
        .ok_or_else(usage)?;
    let mut parsed = NameArgs {
        name: name.clone(),
        owner: None,
        contains: false,
    };
    while let Some(flag) = rest.next() {
        match flag.as_str() {
            "--owner" => parsed.owner = Some(rest.next().ok_or_else(usage)?.clone()),
            "--contains" => parsed.contains = true,
            _ => return Err(usage()),
        }
    }
    if parsed.contains && parsed.owner.is_some() {
        return Err(usage());
    }
    Ok(Some(parsed))
}

#[cfg(test)]
mod tests {
    include!("args_tests.rs");
}
