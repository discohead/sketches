use anyhow::Result;
use clap::{Parser, Subcommand};
use studio::workspace::Workspace;
use studio::{capture, doctor, media, new, pieces};

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
    /// Render frames of a piece and print the path of a contact sheet
    Look {
        piece: String,
        /// Times in seconds, comma separated (default: evenly spaced)
        #[arg(long, value_delimiter = ',')]
        at: Vec<f32>,
        /// How many evenly spaced frames to render when --at is not given
        #[arg(long, default_value_t = 6)]
        count: usize,
    },
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
        Cmd::Look { piece, at, count } => {
            let p = pieces::resolve(&ws, &piece)?;
            let times = if at.is_empty() {
                if count == 0 {
                    anyhow::bail!("--count must be at least 1");
                }
                capture::evenly_spaced(&p.meta, count)
            } else {
                at
            };
            let frames = capture::frames_for_times(&times, p.meta.time.fps);
            let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
            let out = ws.root.join(".studio/look").join(&p.folder).join(stamp);
            let paths = capture::render(&ws, &p, &frames, &out)?;
            let sheet = out.join("montage.png");
            media::montage(&ws, &paths, &sheet)?;
            println!("{}", sheet.display());
            for f in &paths {
                println!("{}", f.display());
            }
            Ok(0)
        }
    }
}
