//! `build` subcommand handler.

use anyhow::{Context, Result};
use std::path::PathBuf;
use tracing::info;

use crate::models::resume::Resume;
use crate::templates::engine::TemplateEngine;
use crate::utils::latex::LatexCompiler;
use crate::validation::resume_validator::ResumeValidator;

pub fn run(
    resume_path: &PathBuf,
    format: &str,
    output_dir: &PathBuf,
    template_dir: &PathBuf,
    theme: &str,
) -> Result<()> {
    let resume = Resume::from_file(resume_path)
        .with_context(|| format!("Failed to load resume from {:?}", resume_path))?;

    let validator = ResumeValidator::new();
    let validation_result = validator.validate(&resume)?;

    if !validation_result.is_valid {
        eprintln!("❌ Resume validation failed. Fix errors before building:");
        for error in &validation_result.errors {
            eprintln!("  ❌ {}", error);
        }
        return Err(anyhow::anyhow!("Resume validation failed"));
    }

    let templates = TemplateEngine::new(template_dir)?;

    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {:?}", output_dir))?;

    match format.to_lowercase().as_str() {
        "latex" => {
            info!("Rendering LaTeX...");
            let latex_content = templates.render_latex(&resume)?;
            let latex_path = output_dir.join("resume.tex");
            std::fs::write(&latex_path, latex_content)
                .with_context(|| format!("Failed to write LaTeX file: {:?}", latex_path))?;
            info!("✅ LaTeX written to: {:?}", latex_path);
        }
        "html" => {
            info!("Rendering HTML...");
            let html_content = templates.render_html(&resume, theme)?;
            let html_path = output_dir.join("resume.html");
            std::fs::write(&html_path, html_content)
                .with_context(|| format!("Failed to write HTML file: {:?}", html_path))?;
            info!("✅ HTML written to: {:?}", html_path);
        }
        "pdf" => {
            info!("Rendering and compiling PDF...");
            let latex_content = templates.render_latex(&resume)?;
            let compiler = LatexCompiler::default();
            let pdf_path = compiler.compile_to_pdf(&latex_content, output_dir)?;
            info!("✅ PDF compiled to: {}", pdf_path);
        }
        "text" => {
            info!("Rendering ATS text...");
            let text_content = templates.render_ats_text(&resume)?;
            let text_path = output_dir.join("resume.txt");
            std::fs::write(&text_path, text_content)
                .with_context(|| format!("Failed to write text file: {:?}", text_path))?;
            info!("✅ ATS text written to: {:?}", text_path);
        }
        "all" => {
            info!("Building all formats...");

            let latex_content = templates.render_latex(&resume)?;
            let latex_path = output_dir.join("resume.tex");
            std::fs::write(&latex_path, &latex_content)
                .with_context(|| format!("Failed to write LaTeX file: {:?}", latex_path))?;

            let html_content = templates.render_html(&resume, theme)?;
            let html_path = output_dir.join("resume.html");
            std::fs::write(&html_path, &html_content)
                .with_context(|| format!("Failed to write HTML file: {:?}", html_path))?;

            let compiler = LatexCompiler::default();
            let pdf_path = compiler.compile_to_pdf(&latex_content, output_dir)?;

            let ats_content = templates.render_ats_text(&resume)?;
            let ats_path = output_dir.join("resume-ats.txt");
            std::fs::write(&ats_path, ats_content)?;

            info!("✅ All formats built:");
            info!("  LaTeX: {:?}", latex_path);
            info!("  HTML:  {:?}", html_path);
            info!("  PDF:   {}", pdf_path);
            info!("  ATS:   {:?}", ats_path);
        }
        _ => {
            return Err(anyhow::anyhow!("Unsupported format: {}", format));
        }
    }

    Ok(())
}
