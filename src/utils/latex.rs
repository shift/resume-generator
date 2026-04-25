use anyhow::{Context, Result};
use std::convert::AsRef;
use std::path::Path;
use std::process::Command;

pub struct LatexCompiler {
    engine: LatexEngine,
    shell_escape: bool,
}

#[derive(Debug, Clone)]
pub enum LatexEngine {
    Tectonic,
    Xelatex,
    Latexmk,
}

impl LatexCompiler {
    pub fn new(engine: LatexEngine) -> Self {
        Self {
            engine,
            shell_escape: false,
        }
    }

    /// Compile LaTeX source to PDF
    pub fn compile_to_pdf<P: AsRef<Path>>(
        &self,
        tex_source: &str,
        output_dir: P,
    ) -> Result<String> {
        let output_dir = output_dir.as_ref();

        // Ensure output directory exists
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)
                .with_context(|| format!("Failed to create output directory: {:?}", output_dir))?;
        }

        // Write temporary LaTeX file
        let tex_path = output_dir.join("resume.tex");
        crate::templates::engine::write_output(tex_source, &tex_path)?;

        match &self.engine {
            LatexEngine::Tectonic => self.compile_with_tectonic(&tex_path, output_dir),
            LatexEngine::Xelatex => self.compile_with_xelatex(tex_source, &tex_path, output_dir),
            LatexEngine::Latexmk => self.compile_with_latexmk(tex_source, &tex_path, output_dir),
        }
    }

    fn compile_with_tectonic(&self, tex_path: &Path, output_dir: &Path) -> Result<String> {
        let output = Command::new("tectonic")
            .arg("--outdir")
            .arg(output_dir)
            .arg(tex_path)
            .output()
            .context("Failed to execute tectonic")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Tectonic compilation failed: {}", stderr));
        }

        let pdf_path = output_dir.join("resume.pdf");
        let pdf_path_str = pdf_path.to_string_lossy().to_string();

        if !pdf_path.exists() {
            return Err(anyhow::anyhow!(
                "LaTeX compiler reported success but PDF was not produced at {}",
                pdf_path_str
            ));
        }

        Ok(pdf_path_str)
    }

    fn compile_with_xelatex(
        &self,
        _tex_source: &str,
        tex_path: &Path,
        output_dir: &Path,
    ) -> Result<String> {
        let mut cmd = Command::new("xelatex");
        cmd.arg("-output-directory")
            .arg(output_dir)
            .arg("-interaction=nonstopmode");

        if self.shell_escape {
            cmd.arg("-shell-escape");
        }

        cmd.arg(tex_path);

        let output = cmd.output().context("Failed to execute xelatex")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("XeLaTeX compilation failed: {}", stderr));
        }

        let pdf_path = output_dir.join("resume.pdf");
        let pdf_path_str = pdf_path.to_string_lossy().to_string();

        if !pdf_path.exists() {
            return Err(anyhow::anyhow!(
                "LaTeX compiler reported success but PDF was not produced at {}",
                pdf_path_str
            ));
        }

        Self::cleanup_aux_files(output_dir, "resume");

        Ok(pdf_path_str)
    }

    fn compile_with_latexmk(
        &self,
        _tex_source: &str,
        tex_path: &Path,
        output_dir: &Path,
    ) -> Result<String> {
        let mut cmd = Command::new("latexmk");
        cmd.arg("-pdf").arg("-interaction=nonstopmode");

        if self.shell_escape {
            cmd.arg("-shell-escape");
        }

        cmd.arg("-output-directory").arg(output_dir).arg(tex_path);

        let output = cmd.output().context("Failed to execute latexmk")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Latexmk compilation failed: {}", stderr));
        }

        let pdf_path = output_dir.join("resume.pdf");
        let pdf_path_str = pdf_path.to_string_lossy().to_string();

        if !pdf_path.exists() {
            return Err(anyhow::anyhow!(
                "LaTeX compiler reported success but PDF was not produced at {}",
                pdf_path_str
            ));
        }

        Self::cleanup_aux_files(output_dir, "resume");

        Ok(pdf_path_str)
    }

    /// Remove auxiliary files produced during LaTeX compilation.
    fn cleanup_aux_files(output_dir: &Path, base: &str) {
        for ext in &["aux", "log", "fls", "fdb_latexmk", "out"] {
            let path = output_dir.join(format!("{}.{}", base, ext));
            let _ = std::fs::remove_file(&path);
        }
    }

    /// Check if LaTeX engine is available
    pub fn check_engine_available(&self) -> bool {
        let cmd = match &self.engine {
            LatexEngine::Tectonic => "tectonic",
            LatexEngine::Xelatex => "xelatex",
            LatexEngine::Latexmk => "latexmk",
        };

        Command::new(cmd)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl Default for LatexCompiler {
    fn default() -> Self {
        // Try engines in order of preference
        let engines = vec![
            LatexEngine::Tectonic,
            LatexEngine::Xelatex,
            LatexEngine::Latexmk,
        ];

        for engine in engines {
            let compiler = LatexCompiler::new(engine.clone());
            if compiler.check_engine_available() {
                return compiler;
            }
        }

        // Fallback to latexmk (most common)
        LatexCompiler::new(LatexEngine::Latexmk)
    }
}
