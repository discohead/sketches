use anyhow::Result;
use clap::{Parser, Subcommand};
use studio::workspace::Workspace;
use studio::{doctor, new, pieces};

#[derive(Parser)]
#[command(name = "studio", about = "Create, list and run art pieces")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Check the tools and folders the studio depends on
    Doctor,
    /// Create a new piece in pieces/<date>-<slug>
    New {
        slug: String,
        /// Start from a copy of this piece (folder, slug or catalog number)
        #[arg(long)]
        from: Option<String>,
    },
    /// List all pieces
    List,
    /// Run a piece in a window
    Run { piece: String },
}

fn main() {
    match real_main() {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::exit(1);
        }
    }
}

fn real_main() -> Result<i32> {
    let cli = Cli::parse();
    let ws = Workspace::discover(&std::env::current_dir()?)?;
    match cli.command {
        Cmd::Doctor => Ok(if doctor::run_all(&ws) { 0 } else { 1 }),
        Cmd::New { slug, from } => {
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            let piece = new::create(&ws, &slug, from.as_deref(), &today)?;
            println!("{}", piece.dir.display());
            Ok(0)
        }
        Cmd::List => {
            for p in pieces::all(&ws)? {
                let title = if p.meta.title.trim().is_empty() { "(untitled)" } else { &p.meta.title };
                println!(
                    "{}  {}  {}  {}",
                    p.folder,
                    p.state(),
                    title,
                    p.meta.series.as_deref().unwrap_or("")
                );
            }
            Ok(0)
        }
        Cmd::Run { piece } => {
            let p = pieces::resolve(&ws, &piece)?;
            let status = ws.cargo().args(["run", "-p", &p.package]).status()?;
            Ok(status.code().unwrap_or(1))
        }
    }
}
