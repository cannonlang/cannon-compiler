#![deny(clippy::all)]
#![warn(clippy::nursery, clippy::pedantic)]
#![feature(if_let_guard)]

pub mod error;
pub mod lex;
pub mod span;

use std::{path::PathBuf, process, sync::RwLock};

use clap::{error::ErrorKind, CommandFactory, Parser};
use error::CannonError;

/// Official compiler for the Cannon programming language
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    /// Input files
    files: Vec<String>,

    /// Output file name, if any
    #[arg(short)]
    output: Option<String>,

    /// Compile only, don't link
    #[arg(short)]
    compile_only: bool,

    /// Highlight only, don't compile
    #[arg(long)]
    highlight_only: bool,
}

static CURRENT_FILE: RwLock<String> = RwLock::new(String::new());

fn main() {
    if let Err(e) = run_frontend() {
        let file_text = CURRENT_FILE.read().unwrap();
        let file_text = file_text.clone();
        let file_lines: Vec<_> = file_text.lines().collect();
        match e {
            CannonError::Eof(pos) => {
                println!("{}", file_lines[pos.0 - 1]);
                println!("{}^ unexpected EOF", " ".repeat(pos.1 - 1))
            }
            CannonError::UnexpectedChar(c, pos) => {
                println!("{}", file_lines[pos.0 - 1]);
                println!("{}^ unexpected {:?}", " ".repeat(pos.1 - 1), c)
            }
            _ => eprintln!("{}", e),
        }
        process::exit(1);
    }
}

fn run_frontend() -> Result<(), CannonError> {
    let options = Options::parse();
    if options.output.is_some() && options.files.len() > 1 && options.compile_only {
        Options::command()
            .error(
                ErrorKind::ArgumentConflict,
                "output specified along with multiple input files in compile-only mode; this is not allowed",
            )
            .exit();
    }
    if options.compile_only {
        for file in &options.files {
            let output = options.output.clone().unwrap_or_else(|| {
                file.strip_suffix(".cannon").unwrap_or(&file).to_string() + ".o"
            });
            let file = PathBuf::from(file);
            let output = output.into();
            if !file.exists() {
                Options::command()
                    .error(
                        ErrorKind::Io,
                        &format!("file `{}` not found", file.display()),
                    )
                    .exit();
            }
            compile(file, output)?;
        }
    }
    if options.highlight_only {
        for file in &options.files {
            let output = options.output.clone().unwrap_or_else(|| {
                file.strip_suffix(".cannon").unwrap_or(&file).to_string() + ".o"
            });
            let file = PathBuf::from(file);
            let output = output.into();
            if !file.exists() {
                Options::command()
                    .error(
                        ErrorKind::Io,
                        &format!("file `{}` not found", file.display()),
                    )
                    .exit();
            }
            highlight(file, output)?;
        }
    }
    Ok(())
}

fn compile(file: PathBuf, _output: PathBuf) -> Result<(), CannonError> {
    let file_str = std::fs::read_to_string(file)?;
    *CURRENT_FILE.write().unwrap() = file_str.clone();
    let lexed = lex::lex(file_str.chars())?;
    println!("{:#?}", lexed);
    Ok(())
}

fn highlight(file: PathBuf, _output: PathBuf) -> Result<(), CannonError> {
    let file_str = std::fs::read_to_string(file)?;
    *CURRENT_FILE.write().unwrap() = file_str.clone();
    let lexed = lex::lex(file_str.chars())?;
    println!("{}", lex::highlight(&lexed));
    Ok(())
}
