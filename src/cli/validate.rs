//! `validate` subcommand handler.

use anyhow::{Context, Result};
use std::path::PathBuf;
use tracing::info;

use crate::models::resume::Resume;
use crate::validation::resume_validator::ResumeValidator;

pub fn run(resume_path: &PathBuf) -> Result<()> {
    let resume = Resume::from_file(resume_path)
        .with_context(|| format!("Failed to load resume from {:?}", resume_path))?;

    let validator = ResumeValidator::new();
    let result = validator.validate(&resume)?;

    if result.is_valid {
        info!("✅ Resume is valid!");
        if result.warnings.is_empty() {
            println!("Resume validation passed with no warnings.");
        } else {
            println!("Resume validation passed with warnings:");
            for warning in &result.warnings {
                println!("  ⚠️  {}", warning);
            }
        }
    } else {
        eprintln!("❌ Resume validation failed:");
        for error in &result.errors {
            eprintln!("  ❌ {}", error);
        }
        return Err(anyhow::anyhow!("Resume validation failed"));
    }

    Ok(())
}
