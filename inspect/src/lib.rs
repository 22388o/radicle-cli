use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use rad_common::{git, profile, project};
use rad_terminal::args::{Args, Error, Help};
use rad_terminal::components as term;

use librad::git::identities::any;
use librad::git::Urn;

use anyhow::anyhow;

use colored_json::prelude::*;

pub const HELP: Help = Help {
    name: "inspect",
    description: env!("CARGO_PKG_DESCRIPTION"),
    version: env!("CARGO_PKG_VERSION"),
    usage: r#"
Usage

    rad inspect <path> [<option>...]
    rad inspect <urn> [<option>...]
    rad inspect

    Inspects the given path or URN. If neither is specified,
    the current directory is inspected.

Options

    --help   Print help
"#,
};

#[derive(Default, Debug, Eq, PartialEq)]
pub struct Options {
    pub path: Option<PathBuf>,
    pub urn: Option<Urn>,
}

impl Args for Options {
    fn from_args(args: Vec<OsString>) -> anyhow::Result<(Self, Vec<OsString>)> {
        use lexopt::prelude::*;

        let mut parser = lexopt::Parser::from_args(args);
        let mut path: Option<PathBuf> = None;
        let mut urn: Option<Urn> = None;

        while let Some(arg) = parser.next()? {
            match arg {
                Long("help") => {
                    return Err(Error::Help.into());
                }
                Value(val) if path.is_none() && urn.is_none() => {
                    let val = val.to_string_lossy();

                    if let Ok(val) = Urn::from_str(&val) {
                        urn = Some(val);
                    } else if val.starts_with("rad:git:") {
                        return Err(anyhow!("invalid URN '{}'", val));
                    } else if let Ok(val) = PathBuf::from_str(&val) {
                        path = Some(val);
                    } else {
                        return Err(anyhow!("invalid path or URN '{}'", val));
                    }
                }
                _ => return Err(anyhow::anyhow!(arg.unexpected())),
            }
        }

        Ok((Options { path, urn }, vec![]))
    }
}

pub fn run(options: Options) -> anyhow::Result<()> {
    let profile = profile::default()?;
    let storage = profile::read_only(&profile)?;

    if let Some(urn) = options.urn {
        let payload = any::get(&storage, &urn)
            .map(|o| o.map(|p| p.payload()))
            .map_err(|_| anyhow::anyhow!("Couldn't load project or person."))?
            .ok_or(anyhow::anyhow!("No project or person found for this URN"))?;

        println!(
            "{}",
            serde_json::to_string_pretty(&payload)?.to_colored_json_auto()?
        );
    } else {
        let repo =
            git::Repository::open(options.path.unwrap_or_else(|| Path::new(".").to_path_buf()))?;
        let urn = project::remote(&repo)?.url.urn;

        term::info!("{}", term::format::highlight(urn));
    }

    Ok(())
}