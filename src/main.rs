use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

mod cli;
mod models;
mod templates;
mod utils;
mod validation;

#[derive(Parser)]
#[command(name = "resume-builder")]
#[command(about = "A fast, safe JSON Resume builder written in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate resume JSON against schema
    Validate {
        /// Path to resume JSON file
        #[arg(short, long, default_value = "resume.json")]
        resume: PathBuf,
    },
    /// Build resume in specified format
    Build {
        /// Path to resume JSON file
        #[arg(short, long, default_value = "resume.json")]
        resume: PathBuf,
        /// Output format (latex, html, pdf, text, all)
        #[arg(short, long, default_value = "pdf")]
        format: String,
        /// Output directory
        #[arg(short, long, default_value = "build")]
        output: PathBuf,
        /// Directory containing Tera templates
        #[arg(long, default_value = "templates")]
        template_dir: PathBuf,
        /// HTML theme (modern, clean-minimal, modern-professional, tech-timeline)
        #[arg(short = 't', long, default_value = "modern")]
        theme: String,
    },
    /// Extract keywords from resume for ATS optimisation
    Keywords {
        /// Path to resume JSON file
        #[arg(short, long, default_value = "resume.json")]
        resume: PathBuf,
    },
    /// Initialise a new resume project
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { resume } => {
            info!("Validating resume: {:?}", resume);
            cli::validate::run(&resume)?;
        }
        Commands::Build {
            resume,
            format,
            output,
            template_dir,
            theme,
        } => {
            info!("Building resume: {:?} -> {:?} ({})", resume, output, format);
            cli::build::run(&resume, &format, &output, &template_dir, &theme)?;
        }
        Commands::Keywords { resume } => {
            info!("Extracting keywords from: {:?}", resume);
            cli::keywords::run(&resume)?;
        }
        Commands::Init { name } => {
            info!("Initialising new resume project");
            cli::init::run(name)?;
        }
    }

    Ok(())
}
