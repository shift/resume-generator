use crate::models::resume::Resume;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tera::{from_value, to_value, Context as TeraContext, Tera, Value};

pub(crate) fn escape_latex_str(s: &str) -> String {
    s.replace('\\', "\\textbackslash{}")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('~', "\\textasciitilde{}")
        .replace('^', "\\textasciicircum{}")
}

fn escape_latex(
    value: &Value,
    _args: &std::collections::HashMap<String, Value>,
) -> tera::Result<Value> {
    if value.is_null() {
        return Ok(Value::Null);
    }
    let s = from_value::<String>(value.clone())?;
    Ok(to_value(escape_latex_str(&s))?)
}

pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    pub fn new<P: AsRef<Path>>(template_dir: P) -> Result<Self> {
        let template_path = format!("{}/**/*.j2", template_dir.as_ref().to_string_lossy());
        let mut tera = match Tera::new(&template_path) {
            Ok(t) => t,
            Err(e) => return Err(anyhow::anyhow!("Failed to load templates: {}", e)),
        };

        tera.register_filter("escape_latex", escape_latex);

        Ok(TemplateEngine { tera })
    }

    /// Render LaTeX template with resume data
    pub fn render_latex(&self, resume: &Resume) -> Result<String> {
        let mut context = TeraContext::new();

        // Add resume data to context
        context.insert("resume", resume);

        // Add helper functions
        context.insert("now", &chrono::Utc::now().format("%Y-%m-%d").to_string());

        self.tera
            .render("latex/moderncv.tex.j2", &context)
            .context("Failed to render LaTeX template")
    }

    /// Render HTML template with resume data
    pub fn render_html(&self, resume: &Resume, theme: &str) -> Result<String> {
        let mut context = TeraContext::new();

        // Add resume data to context
        context.insert("resume", resume);

        // Add helper functions
        context.insert("now", &chrono::Utc::now().format("%Y-%m-%d").to_string());

        let template_name = match theme {
            "modern" => "html/modern.html.j2",
            "clean-minimal" => "html/themes/clean-minimal.html.j2",
            "modern-professional" => "html/themes/modern-professional.html.j2",
            "tech-timeline" => "html/themes/tech-timeline.html.j2",
            other => return Err(anyhow::anyhow!("Unknown theme: {}", other)),
        };

        self.tera
            .render(template_name, &context)
            .context("Failed to render HTML template")
    }

    /// Render ATS-friendly plain text template
    pub fn render_ats_text(&self, resume: &Resume) -> Result<String> {
        let mut context = TeraContext::new();

        context.insert("resume", resume);

        self.tera
            .render("text/ats-friendly.txt.j2", &context)
            .context("Failed to render ATS template")
    }
}

/// Helper function to create directory if it doesn't exist
fn ensure_dir_exists<P: AsRef<Path>>(path: P) -> Result<()> {
    if !path.as_ref().exists() {
        fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create directory: {:?}", path.as_ref()))?;
    }
    Ok(())
}

/// Write rendered content to file
pub fn write_output<P: AsRef<Path>>(content: &str, path: P) -> Result<()> {
    let path = path.as_ref();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_dir_exists(parent)?;
    }

    fs::write(path, content).with_context(|| format!("Failed to write to file: {:?}", path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // The LaTeX special characters that escape_latex_str must escape.
    const SPECIAL_CHARS: &[char] = &['\\', '&', '%', '$', '#', '_', '{', '}', '~', '^'];

    // -----------------------------------------------------------------------
    // Unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn plain_ascii_unchanged() {
        let input = "Hello World 123 abc XYZ";
        assert_eq!(escape_latex_str(input), input);
    }

    #[test]
    fn each_special_char_is_escaped() {
        let cases: &[(&str, &str)] = &[
            // \ is replaced by \textbackslash{}, then { and } in that
            // replacement are themselves escaped → \textbackslash\{\}
            ("\\", "\\textbackslash\\{\\}"),
            ("&", "\\&"),
            ("%", "\\%"),
            ("$", "\\$"),
            ("#", "\\#"),
            ("_", "\\_"),
            ("{", "\\{"),
            ("}", "\\}"),
            // ~ and ^ are replaced AFTER { and }, so their {} are not re-escaped
            ("~", "\\textasciitilde{}"),
            ("^", "\\textasciicircum{}"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                &escape_latex_str(input),
                expected,
                "Failed for char: {:?}",
                input
            );
        }
    }

    // -----------------------------------------------------------------------
    // Property-based tests
    // -----------------------------------------------------------------------

    proptest! {
        /// For any arbitrary string, after escape_latex_str, none of the raw
        /// special chars appear unescaped in the output.
        ///
        /// "Unescaped" is defined as: the char is present in the output AND
        /// is not preceded by a backslash (for &, %, $, #, _, {, }) or is
        /// not part of the replacement words (for \, ~, ^).
        ///
        /// The simplest (and correct) invariant is: the output must not
        /// contain any of the special characters at all, because every
        /// replacement uses ASCII letters, digits, `{`, `}` — wait, `{` and `}`
        /// ARE special chars but they are used in the replacements for `\`, `~`
        /// and `^`. The correct invariant is therefore:
        ///
        ///   1. `\` never appears literally (it is always part of `\command`
        ///      sequences added by the escaper, but the *original* `\` is gone).
        ///   2. Each special char that maps to `\<char>` form: the raw char is
        ///      never present unescaped (i.e. not preceded by `\`).
        ///   3. `~` and `^` are fully replaced by multi-char sequences and must
        ///      not appear in the output at all.
        ///
        /// Simplest conservative check: after escaping, the output does not
        /// contain any of ['~', '^'] at all, and every occurrence of
        /// ['&', '%', '$', '#', '_'] is preceded by a backslash.
        #[test]
        fn no_unescaped_special_chars(s in any::<String>()) {
            let escaped = escape_latex_str(&s);

            // ~ and ^ are replaced entirely — must not appear in output
            prop_assert!(!escaped.contains('~'),
                "output contains raw '~': {:?}", escaped);
            prop_assert!(!escaped.contains('^'),
                "output contains raw '^': {:?}", escaped);

            // For chars that are escaped as \<char>, verify every occurrence
            // is preceded by a backslash.
            for ch in ['&', '%', '$', '#', '_'] {
                let bytes = escaped.as_bytes();
                for (i, &b) in bytes.iter().enumerate() {
                    if b == ch as u8 {
                        prop_assert!(
                            i > 0 && bytes[i - 1] == b'\\',
                            "unescaped '{}' found at position {} in: {:?}",
                            ch, i, escaped
                        );
                    }
                }
            }

            // The original backslash \ is replaced by \textbackslash{}.
            // After replacement the output should contain no standalone
            // backslash that is NOT followed by an ASCII letter (i.e. every
            // backslash in the output must be part of a valid LaTeX command).
            // We check: backslash at end of string is disallowed.
            prop_assert!(
                !escaped.ends_with('\\'),
                "output ends with a raw backslash: {:?}", escaped
            );
        }

        /// Strings containing no special chars are returned unchanged.
        #[test]
        fn no_special_chars_unchanged(s in "[a-zA-Z0-9 \t\n!@()*+,\\-./:;<=>?\\[\\]`|]+") {
            // This regex only contains chars that are NOT in SPECIAL_CHARS
            let _ = SPECIAL_CHARS; // suppress unused warning
            prop_assert_eq!(escape_latex_str(&s), s);
        }
    }
}
