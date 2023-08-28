mod ffi;
mod nix;
mod nix_raw;

use anyhow::{anyhow, bail};
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

type Result<T> = anyhow::Result<T>;

fn main() -> Result<()> {
    let mut args = std::env::args().collect::<Vec<_>>();
    let expr: String = match args.len() {
        1 => {
            let mut stdin = std::io::stdin();
            let mut contents = String::new();
            stdin.read_to_string(&mut contents)?;
            Ok(contents)
        }
        2 => {
            let path = PathBuf::from(args[1].clone());
            let contents = std::fs::read_to_string(path)?;
            Ok(contents)
        }
        3 => {
            let flag = args.get(1).unwrap();
            if flag == "-e" {
                Ok::<String, anyhow::Error>(args[2].clone())
            } else {
                bail!("unrecognized flag {}", flag)
            }
        }
        _ => {
            print_usage();
            bail!("unrecognized arguments");
        }
    }?;
    // let output = nix_raw::eval_string_raw_ffi(expr)?;
    let output = nix::eval_string(expr)?;
    println!("{}", output);
    Ok(())
}

/// Print the usage string for the program
fn print_usage() {
    let usage = r#"
Usage: nix-bindgen (| <FILE> | -e <EXPR>)

When no arguments are provided the expression will be read from stdin.
    "#;
    println!("{usage}");
}
