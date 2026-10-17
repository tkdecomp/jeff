use std::io::{BufRead, Write};

use anyhow::{Result, anyhow};
use argp::FromArgs;
use cwdemangle::{DemangleOptions, demangle};

#[derive(FromArgs, PartialEq, Eq, Debug)]
/// Demangle a CodeWarrior or MSVC C++ symbol.
#[argp(subcommand, name = "demangle")]
pub struct Args {
    #[argp(positional)]
    /// symbol to demangle; without it, one symbol per line from stdin (a line that can't be
    /// demangled is printed unchanged)
    symbol: Option<String>,
    #[argp(switch)]
    /// disable replacing `(void)` with `()`
    keep_void: bool,
    /// enable Metrowerks extensions
    #[argp(switch)]
    mw_extensions: bool,
}

pub fn run(args: Args) -> Result<()> {
    let options = DemangleOptions {
        omit_empty_parameters: !args.keep_void,
        mw_extensions: args.mw_extensions,
    };
    let Some(symbol) = &args.symbol else {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        for line in std::io::stdin().lock().lines() {
            let line = line?;
            writeln!(out, "{}", demangle_any(&line, &options).unwrap_or(line))?;
        }
        return Ok(());
    };
    match demangle_any(symbol, &options) {
        Some(symbol) => {
            println!("{symbol}");
            Ok(())
        }
        None => Err(anyhow!("Failed to demangle symbol")),
    }
}

/// MSVC names start with '?'; everything else is tried as CodeWarrior.
fn demangle_any(symbol: &str, options: &DemangleOptions) -> Option<String> {
    if symbol.starts_with('?') {
        msvc_demangler::demangle(symbol, msvc_demangler::DemangleFlags::COMPLETE).ok()
    } else {
        demangle(symbol, options)
    }
}
