mod difflib;
mod entities;
mod find_session;
mod html_parser;
mod page_text;
mod pytext;
mod review;
mod reviewer;
mod unescape;

use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

const PAGE_TEXT_USAGE: &str =
    "usage: mudora page-text page.html | mudora page-text last-reviewed.html now.html";
const FIND_SESSION_USAGE: &str = "usage: mudora find-session <artifact link>";

#[derive(Parser)]
#[command(name = "mudora", version, about = "Review mudora artifact pages")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Review an artifact page in a headless Claude
    Review(Box<review::ReviewArgs>),
    /// Print a page's visible text, or the diff between two pages
    PageText {
        #[arg(value_name = "PAGE", num_args = 0.., allow_hyphen_values = true)]
        pages: Vec<String>,
    },
    /// Find the session that last published an artifact
    FindSession {
        #[arg(value_name = "LINK", num_args = 0.., allow_hyphen_values = true)]
        links: Vec<String>,
    },
}

fn page_text(pages: &[String]) -> Result<String, String> {
    match pages {
        [page] => Ok(page_text::page_lines(Path::new(page))?.join("\n")),
        [old, new] => Ok(page_text::text_diff(Path::new(old), Path::new(new))?.join("\n")),
        _ => Err(PAGE_TEXT_USAGE.into()),
    }
}

fn find_session(links: &[String]) -> Result<String, String> {
    match links {
        [link] => find_session::run(link),
        _ => Err(FIND_SESSION_USAGE.into()),
    }
}

fn main() -> ExitCode {
    let output = match Cli::parse().command {
        Command::Review(args) => review::run(&args),
        Command::PageText { pages } => page_text(&pages),
        Command::FindSession { links } => find_session(&links),
    };
    let printed = output
        .and_then(|text| writeln!(std::io::stdout(), "{text}").map_err(|e| format!("stdout: {e}")));
    match printed {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
