//! `keywords` subcommand handler.

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::models::resume::Resume;

pub fn run(resume_path: &PathBuf) -> Result<()> {
    let resume = Resume::from_file(resume_path)
        .with_context(|| format!("Failed to load resume from {:?}", resume_path))?;

    let keywords = resume.get_all_keywords();

    println!("🔍 Extracted {} keywords:", keywords.len());
    println!();

    for (i, keyword) in keywords.iter().enumerate() {
        println!("{:3}. {}", i + 1, keyword);
    }

    println!();
    println!("📊 Keyword Analysis:");

    let mut keyword_counts = std::collections::HashMap::new();
    for keyword in &keywords {
        *keyword_counts.entry(keyword.clone()).or_insert(0) += 1;
    }

    let mut sorted_keywords: Vec<_> = keyword_counts.iter().collect();
    sorted_keywords.sort_by(|a, b| b.1.cmp(a.1));

    for (keyword, count) in sorted_keywords.iter().take(20) {
        println!("  {} ({} occurrences)", keyword, count);
    }

    Ok(())
}
