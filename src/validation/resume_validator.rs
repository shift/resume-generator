use crate::models::resume::Resume;
use anyhow::Result;
use regex::Regex;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Cached compiled regexes — compiled once, reused for every validation call.
// ---------------------------------------------------------------------------

fn quantification_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Matches any digit sequence (covers plain numbers, percentages, dollar
    // amounts, decimals, and ranges).
    RE.get_or_init(|| Regex::new(r"\d").unwrap())
}

/// Validates a basic `user@domain.tld` structure without false positives.
/// Pattern: `^[^\s@]+@[^\s@]+\.[^\s@]+$`
fn email_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap())
}

/// Build a single word-boundary regex that matches any of the supplied words.
/// Used for both weasel-word and action-verb checks.
fn word_boundary_any(words: &[&str]) -> Regex {
    let alternation = words.join("|");
    Regex::new(&format!(r"(?i)\b(?:{})\b", alternation)).unwrap()
}

pub struct ValidationReport {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub struct ResumeValidator {
    // Schema validation rules and checks
}

impl Default for ResumeValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ResumeValidator {
    pub fn new() -> Self {
        Self {}
    }

    /// Validate resume against JSON Resume schema and best practices
    pub fn validate(&self, resume: &Resume) -> Result<ValidationReport> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Required field validations
        self.validate_basics(&resume.basics, &mut errors, &mut warnings);
        self.validate_work(&resume.work, &mut errors, &mut warnings);
        self.validate_education(&resume.education, &mut errors, &mut warnings);
        self.validate_skills(&resume.skills, &mut errors, &mut warnings);
        self.validate_projects(&resume.projects, &mut errors, &mut warnings);
        self.validate_ats_optimization(resume, &mut warnings);

        // Content quality validations
        self.validate_content_quality(resume, &mut warnings);

        let is_valid = errors.is_empty();

        Ok(ValidationReport {
            is_valid,
            errors,
            warnings,
        })
    }

    fn validate_basics(
        &self,
        basics: &crate::models::resume::Basics,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        // Required fields
        if basics.name.is_empty() {
            errors.push("Name is required in basics".to_string());
        }

        if let Some(email) = &basics.email {
            if !email_regex().is_match(email) {
                errors.push(format!("Invalid email format: {}", email));
            }
        } else {
            warnings.push("Email is recommended for ATS optimization".to_string());
        }

        if let Some(phone) = &basics.phone {
            let phone_digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
            if phone_digits.len() < 10 {
                warnings.push("Phone number appears incomplete".to_string());
            }
        } else {
            warnings.push("Phone number is recommended for ATS optimization".to_string());
        }

        // Summary length check
        if let Some(summary) = &basics.summary {
            if summary.len() < 50 {
                warnings.push(
                    "Summary should be at least 50 characters for professional appearance"
                        .to_string(),
                );
            }
            if summary.len() > 300 {
                warnings.push("Summary should be under 300 characters for impact".to_string());
            }
        } else {
            warnings.push("Summary is highly recommended for professional resumes".to_string());
        }
    }

    fn validate_work(
        &self,
        work: &[crate::models::resume::Work],
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        if work.is_empty() {
            errors.push("At least one work experience entry is required".to_string());
            return;
        }

        for (i, job) in work.iter().enumerate() {
            if job.name.is_empty() {
                errors.push(format!("Work[{}]: company name is required", i));
            }
            if job.position.is_empty() {
                errors.push(format!("Work[{}]: position is required", i));
            }

            // Date validation
            if let (Some(start), Some(end)) = (&job.start_date, &job.end_date) {
                if end != "Present" && self.is_date_after(start, end) {
                    errors.push(format!("Work[{}]: end date cannot be before start date", i));
                }
            }

            // Content quality
            if let Some(summary) = &job.summary {
                if summary.len() > 500 {
                    warnings.push(format!(
                        "Work[{}]: summary should be under 500 characters for readability",
                        i
                    ));
                }
            }

            // Check for action verbs in highlights
            for (j, highlight) in job.highlights.iter().enumerate() {
                if !self.has_action_verb(highlight) {
                    warnings.push(format!(
                        "Work[{}].highlight[{}]: Consider starting with an action verb (e.g., 'Led', 'Delivered', 'Architected')",
                        i, j
                    ));
                }
            }
        }
    }

    fn validate_education(
        &self,
        education: &[crate::models::resume::Education],
        errors: &mut Vec<String>,
        _warnings: &mut Vec<String>,
    ) {
        for (i, edu) in education.iter().enumerate() {
            if edu.institution.is_empty() {
                errors.push(format!("Education[{}]: institution is required", i));
            }

            // Guard against "Present" / non-year sentinels before date comparison.
            // is_date_after returns false when either year cannot be parsed, so
            // "Present" already happens to be safe — but the explicit guard makes
            // the intent clear and handles any future sentinel strings.
            if let (Some(start), Some(end)) = (&edu.start_date, &edu.end_date) {
                if end != "Present" && self.is_date_after(start, end) {
                    errors.push(format!(
                        "Education[{}]: end date cannot be before start date",
                        i
                    ));
                }
            }
        }
    }

    fn validate_skills(
        &self,
        skills: &[crate::models::resume::Skill],
        _errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        if skills.is_empty() {
            warnings.push("Skills section is recommended for ATS optimization".to_string());
            return;
        }

        // Check for duplicate skills
        let mut skill_names = Vec::new();
        for (i, skill) in skills.iter().enumerate() {
            if skill_names.contains(&skill.name.to_lowercase()) {
                warnings.push(format!(
                    "Skills[{}]: duplicate skill category: {}",
                    i, skill.name
                ));
            }
            skill_names.push(skill.name.to_lowercase());
        }

        // Check keyword quality
        for (i, skill) in skills.iter().enumerate() {
            if skill.keywords.is_empty() {
                warnings.push(format!(
                    "Skills[{}]: category '{}' has no keywords",
                    i, skill.name
                ));
            }

            for keyword in &skill.keywords {
                if keyword.len() < 2 {
                    warnings.push(format!("Skills[{}]: keyword '{}' is too short", i, keyword));
                }
            }
        }
    }

    fn validate_projects(
        &self,
        projects: &[crate::models::resume::Project],
        _errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        for (i, project) in projects.iter().enumerate() {
            if project.name.is_empty() {
                warnings.push(format!("Projects[{}]: project name is recommended", i));
            }

            if let Some(description) = &project.description {
                if description.len() > 1000 {
                    warnings.push(format!(
                        "Projects[{}]: description should be under 1000 characters",
                        i
                    ));
                }
            }
        }
    }

    fn validate_ats_optimization(&self, resume: &Resume, warnings: &mut Vec<String>) {
        // Check for ATS-friendly content
        let keywords = resume.get_all_keywords();

        if keywords.len() < 20 {
            warnings.push("Consider adding more keywords for better ATS optimization".to_string());
        }

        // Check for quantified achievements
        let mut quantified_highlights = 0;
        for job in &resume.work {
            for highlight in &job.highlights {
                if self.has_quantification(highlight) {
                    quantified_highlights += 1;
                }
            }
        }

        let total_highlights: usize = resume.work.iter().map(|w| w.highlights.len()).sum();

        if total_highlights > 0 {
            let quantification_ratio = quantified_highlights as f64 / total_highlights as f64;
            if quantification_ratio < 0.5 {
                warnings.push(format!(
                    "Only {}% of achievements are quantified. Consider adding metrics and numbers for impact (e.g., 'Reduced costs by 35%', 'Led team of 5 engineers')",
                    (quantification_ratio * 100.0) as u32
                ));
            }
        } else {
            warnings.push(
                "Consider adding quantified achievements with metrics and numbers".to_string(),
            );
        }
    }

    fn validate_content_quality(&self, resume: &Resume, warnings: &mut Vec<String>) {
        self.check_weasel_words(resume, warnings);
        self.check_passive_voice(resume, warnings);
        self.check_length_optimization(resume, warnings);
    }

    /// Check for weasel words using whole-word regex matching.
    ///
    /// Previously used `String::contains` which caused false positives on
    /// substrings — e.g. "very" matched "delivery", "recovery", "every".
    fn check_weasel_words(&self, resume: &Resume, warnings: &mut Vec<String>) {
        let weasel_words = &[
            "basically",
            "simply",
            "probably",
            "might",
            "could",
            "perhaps",
            "very",
            "really",
            "quite",
            "somewhat",
            "fairly",
            "significantly",
            "various",
            "numerous",
        ];

        let re = word_boundary_any(weasel_words);
        let all_text = self.get_all_text_content(resume);

        let found: Vec<&str> = weasel_words
            .iter()
            .filter(|&&w| {
                // Re-check each word individually so we can report exactly which
                // ones are present, rather than emitting a single bulk match.
                let word_re = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(w))).unwrap();
                word_re.is_match(&all_text)
            })
            .copied()
            .collect();

        // Drop the bulk regex — it was only used to short-circuit when nothing
        // matches at all (avoiding the per-word loop entirely).
        if re.is_match(&all_text) && !found.is_empty() {
            warnings.push(format!(
                "Consider removing weasel words: {}",
                found.join(", ")
            ));
        }
    }

    /// Check for passive voice indicators and report every occurrence found,
    /// not just the first one.
    fn check_passive_voice(&self, resume: &Resume, warnings: &mut Vec<String>) {
        let passive_indicators = &[
            "was responsible for",
            "was tasked with",
            "helped with",
            "assisted in",
            "participated in",
            "involved in",
        ];

        let all_text = self.get_all_text_content(resume);
        let all_text_lower = all_text.to_lowercase();

        for phrase in passive_indicators {
            if all_text_lower.contains(phrase) {
                warnings.push(format!(
                    "Consider using active voice instead of: '{}'",
                    phrase
                ));
            }
        }
    }

    fn check_length_optimization(&self, resume: &Resume, warnings: &mut Vec<String>) {
        let total_text = self.get_all_text_content(resume);
        let word_count = total_text.split_whitespace().count();

        // A single-page resume typically runs 300-600 words.
        // A full CV with projects, multiple roles, and skills is legitimately
        // 1000-2500 words. Only warn at extremes.
        if word_count < 150 {
            warnings.push(format!(
                "Resume is {} words — content appears very thin. Consider expanding.",
                word_count
            ));
        } else if word_count > 3000 {
            warnings.push(format!(
                "Resume is {} words — consider trimming for readability.",
                word_count
            ));
        }
    }

    /// Collect all human-readable text from the resume for content-quality checks.
    ///
    /// Covers basics label/summary, all work, projects, volunteer sections, and
    /// skills keywords to ensure weasel-word and passive-voice checks are thorough.
    fn get_all_text_content(&self, resume: &Resume) -> String {
        let mut parts: Vec<&str> = Vec::new();

        parts.push(&resume.basics.name);
        if let Some(label) = &resume.basics.label {
            parts.push(label.as_str());
        }
        if let Some(s) = &resume.basics.summary {
            parts.push(s.as_str());
        }

        for job in &resume.work {
            parts.push(&job.position);
            parts.push(&job.name);
            if let Some(s) = &job.summary {
                parts.push(s.as_str());
            }
            for h in &job.highlights {
                parts.push(h.as_str());
            }
        }

        for vol in &resume.volunteer {
            if let Some(s) = &vol.summary {
                parts.push(s.as_str());
            }
            for h in &vol.highlights {
                parts.push(h.as_str());
            }
        }

        for skill in &resume.skills {
            for kw in &skill.keywords {
                parts.push(kw.as_str());
            }
        }

        for project in &resume.projects {
            parts.push(&project.name);
            if let Some(d) = &project.description {
                parts.push(d.as_str());
            }
            for h in &project.highlights {
                parts.push(h.as_str());
            }
        }

        parts.join(" ")
    }

    /// Check whether a highlight bullet starts with a recognised action verb.
    ///
    /// Uses a word-boundary regex so that "Deployed" passes but a hypothetical
    /// "Deployedby..." does not. The verb list has been significantly expanded
    /// beyond the original 23 entries to cover common strong resume verbs.
    fn has_action_verb(&self, text: &str) -> bool {
        // The regex anchors at the start of the string (^) and requires a word
        // boundary after the verb so partial prefix matches are rejected.
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| {
            let verbs = [
                // Original set
                "led",
                "managed",
                "developed",
                "implemented",
                "architected",
                "engineered",
                "built",
                "created",
                "designed",
                "launched",
                "established",
                "spearheaded",
                "directed",
                "coordinated",
                "optimized",
                "reduced",
                "increased",
                "improved",
                "automated",
                "streamlined",
                "migrated",
                "deployed",
                "configured",
                // Expanded — previously causing false-positive warnings
                "delivered",
                "prototyped",
                "pioneered",
                "transformed",
                "executed",
                "championed",
                "authored",
                "maintained",
                "researched",
                "defined",
                "owned",
                "drove",
                "accelerated",
                "consolidated",
                "decoupled",
                "eliminated",
                "enabled",
                "enforced",
                "ensured",
                "evaluated",
                "extended",
                "facilitated",
                "generated",
                "guided",
                "hardened",
                "identified",
                "integrated",
                "introduced",
                "led",
                "lifted",
                "mentored",
                "modernised",
                "modernized",
                "monitored",
                "negotiated",
                "onboarded",
                "optimised",
                "overhauled",
                "partnered",
                "planned",
                "ported",
                "presented",
                "prioritised",
                "prioritized",
                "productionised",
                "productionized",
                "proved",
                "provisioned",
                "published",
                "rebuilt",
                "refactored",
                "replaced",
                "resolved",
                "scaled",
                "secured",
                "shipped",
                "simplified",
                "standardised",
                "standardized",
                "triaged",
                "unified",
                "upgraded",
                "validated",
                "wrote",
            ];
            let alternation = verbs.join("|");
            Regex::new(&format!(r"(?i)^(?:{})\b", alternation)).unwrap()
        });

        re.is_match(text)
    }

    /// Returns true if the text contains any digit (covers plain numbers,
    /// percentages, dollar amounts, decimals, and ranges with a single
    /// pre-compiled regex instead of four separate compilations per call).
    fn has_quantification(&self, text: &str) -> bool {
        quantification_regex().is_match(text)
    }

    /// Returns true when `end` is strictly before `start`.
    ///
    /// Performs month-granular comparison when both dates contain a month
    /// component (`YYYY-MM` or `YYYY-MM-DD`). Falls back to year-only comparison
    /// when either date is a bare `YYYY` string.
    fn is_date_after(&self, start: &str, end: &str) -> bool {
        match (self.extract_date_value(start), self.extract_date_value(end)) {
            (Some((sy, Some(sm))), Some((ey, Some(em)))) => {
                // Both have month — compare as YYYY*100+MM integers.
                let start_val = sy * 100 + sm;
                let end_val = ey * 100 + em;
                end_val < start_val
            }
            (Some((sy, _)), Some((ey, _))) => {
                // At least one is year-only — compare years.
                ey < sy
            }
            _ => false,
        }
    }

    /// Parse a date string and return `(year, Option<month>)`.
    ///
    /// Recognised formats (in order of precedence):
    /// - `YYYY-MM-DD` → `(YYYY, Some(MM))`
    /// - `YYYY-MM`    → `(YYYY, Some(MM))`
    /// - `YYYY`       → `(YYYY, None)`
    ///
    /// Any string that does not contain a 4-digit year returns `None`.
    fn extract_date_value(&self, date_str: &str) -> Option<(u32, Option<u32>)> {
        static DATE_RE: OnceLock<Regex> = OnceLock::new();
        let re = DATE_RE.get_or_init(|| {
            // Match YYYY-MM(-DD) or bare YYYY anywhere in the string.
            Regex::new(r"(\d{4})-(\d{2})(?:-\d{2})?|(\d{4})").unwrap()
        });

        let caps = re.captures(date_str)?;
        if let (Some(y), Some(m)) = (caps.get(1), caps.get(2)) {
            let year: u32 = y.as_str().parse().ok()?;
            let month: u32 = m.as_str().parse().ok()?;
            Some((year, Some(month)))
        } else if let Some(y) = caps.get(3) {
            let year: u32 = y.as_str().parse().ok()?;
            Some((year, None))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_resume(name: &str) -> Resume {
        Resume {
            schema: "https://raw.githubusercontent.com/jsonresume/resume-schema/v1.0.0/schema.json"
                .to_string(),
            basics: crate::models::resume::Basics {
                name: name.to_string(),
                label: None,
                image: None,
                email: Some("test@example.com".to_string()),
                phone: None,
                url: None,
                summary: None,
                location: None,
                profiles: vec![],
            },
            work: vec![],
            volunteer: vec![],
            education: vec![],
            awards: vec![],
            certificates: vec![],
            publications: vec![],
            skills: vec![],
            languages: vec![],
            interests: vec![],
            references: vec![],
            projects: vec![],
            meta: None,
        }
    }

    #[test]
    fn test_validation_empty_name() {
        let resume = minimal_resume("");
        let validator = ResumeValidator::new();
        let result = validator.validate(&resume).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("Name is required")));
    }

    // -----------------------------------------------------------------------
    // Action verb tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_has_action_verb_original_verbs() {
        let v = ResumeValidator::new();
        for verb in &[
            "Led",
            "Managed",
            "Developed",
            "Implemented",
            "Architected",
            "Engineered",
            "Built",
            "Created",
            "Designed",
            "Launched",
            "Established",
            "Spearheaded",
            "Directed",
            "Coordinated",
            "Optimized",
            "Reduced",
            "Increased",
            "Improved",
            "Automated",
            "Streamlined",
            "Migrated",
            "Deployed",
            "Configured",
        ] {
            let text = format!("{} something important", verb);
            assert!(
                v.has_action_verb(&text),
                "Expected action verb recognised: {}",
                verb
            );
        }
    }

    #[test]
    fn test_has_action_verb_expanded_verbs() {
        let v = ResumeValidator::new();
        // These previously caused false-positive warnings
        for verb in &[
            "Delivered",
            "Prototyped",
            "Pioneered",
            "Transformed",
            "Executed",
            "Championed",
        ] {
            let text = format!("{} something important", verb);
            assert!(
                v.has_action_verb(&text),
                "Expected expanded verb recognised: {}",
                verb
            );
        }
    }

    #[test]
    fn test_has_action_verb_rejects_non_verbs() {
        let v = ResumeValidator::new();
        assert!(!v.has_action_verb("A cross-functional team of 8 engineers"));
        assert!(!v.has_action_verb("The platform achieved 99.9% uptime"));
        assert!(!v.has_action_verb(""));
    }

    #[test]
    fn test_has_action_verb_word_boundary() {
        let v = ResumeValidator::new();
        // "Deployedby" is not a real word but tests that prefix-only match is rejected
        assert!(!v.has_action_verb("Deployedby something"));
    }

    // -----------------------------------------------------------------------
    // Weasel word tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_weasel_word_no_false_positive_delivery() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        resume.basics.summary =
            Some("Architected a prescription delivery platform with 99.95% uptime.".to_string());
        // "delivery" must NOT trigger the "very" weasel-word warning
        let result = v.validate(&resume).unwrap();
        let weasel_warnings: Vec<_> = result
            .warnings
            .iter()
            .filter(|w| w.contains("weasel"))
            .collect();
        assert!(
            weasel_warnings.is_empty(),
            "False positive weasel word warning from 'delivery': {:?}",
            weasel_warnings
        );
    }

    #[test]
    fn test_weasel_word_no_false_positive_recovery() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        resume.basics.summary = Some("Implemented automated failure recovery systems.".to_string());
        let result = v.validate(&resume).unwrap();
        let weasel_warnings: Vec<_> = result
            .warnings
            .iter()
            .filter(|w| w.contains("weasel"))
            .collect();
        assert!(
            weasel_warnings.is_empty(),
            "False positive weasel word warning from 'recovery': {:?}",
            weasel_warnings
        );
    }

    #[test]
    fn test_weasel_word_detects_real_usage() {
        let v = ResumeValidator::new();
        let resume = {
            let mut r = minimal_resume("Test User");
            r.basics.summary = Some(
                "A very experienced engineer who really excels at basically everything."
                    .to_string(),
            );
            r
        };
        let result = v.validate(&resume).unwrap();
        let weasel_warnings: Vec<_> = result
            .warnings
            .iter()
            .filter(|w| w.contains("weasel"))
            .collect();
        assert!(
            !weasel_warnings.is_empty(),
            "Expected weasel word warning to fire"
        );
        let warning = &weasel_warnings[0];
        assert!(
            warning.contains("very"),
            "Expected 'very' in warning: {}",
            warning
        );
        assert!(
            warning.contains("really"),
            "Expected 'really' in warning: {}",
            warning
        );
        assert!(
            warning.contains("basically"),
            "Expected 'basically' in warning: {}",
            warning
        );
    }

    // -----------------------------------------------------------------------
    // Education date tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_education_present_end_date_no_error() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        resume.education = vec![crate::models::resume::Education {
            institution: "Linux Community".to_string(),
            url: None,
            area: Some("Technical Foundation".to_string()),
            study_type: Some("Self-Directed".to_string()),
            start_date: Some("1994".to_string()),
            end_date: Some("Present".to_string()),
            score: None,
            courses: vec![],
        }];
        let result = v.validate(&resume).unwrap();
        assert!(
            !result
                .errors
                .iter()
                .any(|e| e.contains("end date cannot be before")),
            "Education with 'Present' end date must not produce a date error"
        );
    }

    // -----------------------------------------------------------------------
    // extract_date_value unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn extract_date_value_empty_string_returns_none() {
        let v = ResumeValidator::new();
        assert_eq!(v.extract_date_value(""), None);
    }

    #[test]
    fn extract_date_value_present_returns_none() {
        let v = ResumeValidator::new();
        assert_eq!(v.extract_date_value("Present"), None);
    }

    #[test]
    fn extract_date_value_bare_year_returns_some() {
        let v = ResumeValidator::new();
        assert_eq!(v.extract_date_value("2024"), Some((2024, None)));
    }

    #[test]
    fn extract_date_value_year_month_returns_some() {
        let v = ResumeValidator::new();
        assert_eq!(v.extract_date_value("2024-03"), Some((2024, Some(3))));
    }

    #[test]
    fn extract_date_value_full_date_returns_year_and_month() {
        let v = ResumeValidator::new();
        assert_eq!(v.extract_date_value("2024-01-15"), Some((2024, Some(1))));
    }

    // -----------------------------------------------------------------------
    // is_date_after unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn is_date_after_start_greater_returns_true() {
        let v = ResumeValidator::new();
        assert!(v.is_date_after("2020", "2019"));
    }

    #[test]
    fn is_date_after_equal_years_returns_false() {
        let v = ResumeValidator::new();
        assert!(!v.is_date_after("2020", "2020"));
    }

    #[test]
    fn is_date_after_start_less_than_end_returns_false() {
        let v = ResumeValidator::new();
        assert!(!v.is_date_after("2019", "2020"));
    }

    // -----------------------------------------------------------------------
    // is_date_after — month-granular tests
    // -----------------------------------------------------------------------

    #[test]
    fn is_date_after_same_year_end_month_before_start_month_is_inversion() {
        // Nov 2020 start, Mar 2020 end — end is before start within same year.
        let v = ResumeValidator::new();
        assert!(v.is_date_after("2020-11", "2020-03"));
    }

    #[test]
    fn is_date_after_same_year_equal_months_returns_false() {
        let v = ResumeValidator::new();
        assert!(!v.is_date_after("2020-06", "2020-06"));
    }

    #[test]
    fn is_date_after_same_year_end_month_after_start_month_returns_false() {
        let v = ResumeValidator::new();
        assert!(!v.is_date_after("2020-03", "2020-11"));
    }

    #[test]
    fn is_date_after_full_date_month_granular() {
        // 2021-05-01 start, 2021-02-28 end — inversion within same year.
        let v = ResumeValidator::new();
        assert!(v.is_date_after("2021-05-01", "2021-02-28"));
    }

    #[test]
    fn is_date_after_year_only_vs_year_month_falls_back_to_year() {
        // "2020" vs "2020-06" — start is bare year, compare years only.
        let v = ResumeValidator::new();
        assert!(!v.is_date_after("2020", "2020-06")); // years equal → false
    }

    // -----------------------------------------------------------------------
    // Email validation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_email_valid_formats_no_error() {
        let v = ResumeValidator::new();
        let valid_emails = &[
            "user@example.com",
            "first.last@domain.org",
            "user+tag@sub.domain.co.uk",
            "user123@domain.io",
        ];
        for email in valid_emails {
            let mut resume = minimal_resume("Test User");
            resume.basics.email = Some(email.to_string());
            let result = v.validate(&resume).unwrap();
            assert!(
                !result.errors.iter().any(|e| e.contains("Invalid email")),
                "Valid email '{}' should not produce an error, got: {:?}",
                email,
                result.errors
            );
        }
    }

    #[test]
    fn test_email_invalid_no_at_symbol() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        resume.basics.email = Some("notanemail".to_string());
        let result = v.validate(&resume).unwrap();
        assert!(
            result.errors.iter().any(|e| e.contains("Invalid email")),
            "Missing @ should produce an error, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_email_invalid_no_tld() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        resume.basics.email = Some("user@domain".to_string());
        let result = v.validate(&resume).unwrap();
        assert!(
            result.errors.iter().any(|e| e.contains("Invalid email")),
            "Email without TLD should produce an error, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_email_invalid_spaces() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        resume.basics.email = Some("user @example.com".to_string());
        let result = v.validate(&resume).unwrap();
        assert!(
            result.errors.iter().any(|e| e.contains("Invalid email")),
            "Email with spaces should produce an error, got: {:?}",
            result.errors
        );
    }

    // -----------------------------------------------------------------------
    // Property-based tests
    // -----------------------------------------------------------------------

    use proptest::prelude::*;

    proptest! {
        /// extract_date_value must never panic on any arbitrary string input.
        #[test]
        fn extract_date_value_never_panics(s in any::<String>()) {
            let v = ResumeValidator::new();
            // We only care that this does not panic; result is unconstrained.
            let _ = v.extract_date_value(&s);
        }

        /// is_date_after must never panic on any two arbitrary string inputs.
        #[test]
        fn is_date_after_never_panics(a in any::<String>(), b in any::<String>()) {
            let v = ResumeValidator::new();
            let _ = v.is_date_after(&a, &b);
        }
    }

    // -----------------------------------------------------------------------
    // validate_work — additional coverage
    // -----------------------------------------------------------------------

    fn make_work(name: &str, position: &str) -> crate::models::resume::Work {
        crate::models::resume::Work {
            name: name.to_string(),
            position: position.to_string(),
            url: None,
            start_date: None,
            end_date: None,
            summary: None,
            highlights: vec![],
            location: None,
        }
    }

    #[test]
    fn test_work_date_ordering_error() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        let mut job = make_work("Acme Corp", "Engineer");
        job.start_date = Some("2020".to_string());
        job.end_date = Some("2018".to_string()); // end before start
        resume.work = vec![job];
        let result = v.validate(&resume).unwrap();
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("end date cannot be before start date")),
            "Expected work date ordering error, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_work_summary_length_warning() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        let mut job = make_work("Acme Corp", "Engineer");
        job.summary = Some("x".repeat(501)); // > 500 chars
        resume.work = vec![job];
        let result = v.validate(&resume).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("summary should be under 500 characters")),
            "Expected work summary length warning, got: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_work_highlights_without_action_verbs_warning() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        let mut job = make_work("Acme Corp", "Engineer");
        job.highlights = vec![
            "A cross-functional team collaboration effort".to_string(), // no action verb at start
        ];
        resume.work = vec![job];
        let result = v.validate(&resume).unwrap();
        assert!(
            result.warnings.iter().any(|w| w.contains("action verb")),
            "Expected action verb warning for bullet lacking action verb, got: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_work_highlights_with_action_verb_no_warning() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        let mut job = make_work("Acme Corp", "Engineer");
        job.highlights =
            vec!["Led a team of 5 engineers to deliver the platform on time".to_string()];
        resume.work = vec![job];
        let result = v.validate(&resume).unwrap();
        assert!(
            !result.warnings.iter().any(|w| w.contains("action verb")),
            "Unexpected action verb warning for strong bullet, got: {:?}",
            result.warnings
        );
    }

    // -----------------------------------------------------------------------
    // validate_education — additional coverage
    // -----------------------------------------------------------------------

    fn make_education(institution: &str) -> crate::models::resume::Education {
        crate::models::resume::Education {
            institution: institution.to_string(),
            url: None,
            area: None,
            study_type: None,
            start_date: None,
            end_date: None,
            score: None,
            courses: vec![],
        }
    }

    #[test]
    fn test_education_missing_institution_error() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        resume.education = vec![make_education("")]; // empty institution
        let result = v.validate(&resume).unwrap();
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("institution is required")),
            "Expected missing institution error, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_education_date_ordering_error() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        let mut edu = make_education("MIT");
        edu.start_date = Some("2022".to_string());
        edu.end_date = Some("2019".to_string()); // end before start
        resume.education = vec![edu];
        let result = v.validate(&resume).unwrap();
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("end date cannot be before start date")),
            "Expected education date ordering error, got: {:?}",
            result.errors
        );
    }

    // -----------------------------------------------------------------------
    // validate_skills — coverage
    // -----------------------------------------------------------------------

    fn make_skill(name: &str, keywords: Vec<String>) -> crate::models::resume::Skill {
        crate::models::resume::Skill {
            name: name.to_string(),
            level: None,
            keywords,
        }
    }

    #[test]
    fn test_skills_empty_list_warning() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        resume.skills = vec![]; // present but empty
        let result = v.validate(&resume).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("Skills section is recommended")),
            "Expected empty skills list warning, got: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_skills_duplicate_skill_warning() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        resume.skills = vec![
            make_skill("Rust", vec!["Rust".to_string()]),
            make_skill("Rust", vec!["cargo".to_string()]), // duplicate name
        ];
        let result = v.validate(&resume).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("duplicate skill category")),
            "Expected duplicate skill warning, got: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_skills_short_keyword_warning() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        resume.skills = vec![make_skill(
            "Languages",
            vec!["x".to_string()], // single-char keyword (< 2 chars)
        )];
        let result = v.validate(&resume).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("keyword") && w.contains("too short")),
            "Expected short keyword warning, got: {:?}",
            result.warnings
        );
    }

    // -----------------------------------------------------------------------
    // validate_projects — coverage
    // -----------------------------------------------------------------------

    fn make_project(name: &str, description: Option<String>) -> crate::models::resume::Project {
        crate::models::resume::Project {
            name: name.to_string(),
            description,
            highlights: vec![],
            start_date: None,
            end_date: None,
            url: None,
            roles: vec![],
            entity: None,
            project_type: None,
        }
    }

    #[test]
    fn test_projects_empty_name_warning() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        resume.projects = vec![make_project("", None)];
        let result = v.validate(&resume).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("project name is recommended")),
            "Expected empty project name warning, got: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_projects_description_too_long_warning() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        resume.projects = vec![make_project(
            "MyProject",
            Some("x".repeat(1001)), // > 1000 chars
        )];
        let result = v.validate(&resume).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("description should be under 1000 characters")),
            "Expected project description length warning, got: {:?}",
            result.warnings
        );
    }

    // -----------------------------------------------------------------------
    // validate_ats_optimization — coverage
    // -----------------------------------------------------------------------

    #[test]
    fn test_ats_keyword_count_low_warning() {
        // A minimal resume with no skills and no work has 0 keywords
        let v = ResumeValidator::new();
        let resume = minimal_resume("Test User");
        let result = v.validate(&resume).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("more keywords for better ATS")),
            "Expected low keyword count ATS warning, got: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_ats_quantification_ratio_low_warning() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        let mut job = make_work("Acme Corp", "Engineer");
        // Two highlights, neither contains a digit
        job.highlights = vec![
            "Led team meetings regularly".to_string(),
            "Reviewed pull requests".to_string(),
        ];
        resume.work = vec![job];
        let result = v.validate(&resume).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("quantified") && w.contains("%")),
            "Expected quantification ratio warning, got: {:?}",
            result.warnings
        );
    }

    // -----------------------------------------------------------------------
    // check_passive_voice — coverage
    // -----------------------------------------------------------------------

    #[test]
    fn test_passive_voice_detects_phrase() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        let mut job = make_work("Acme Corp", "Engineer");
        job.summary = Some("Was responsible for maintaining the CI pipeline.".to_string());
        resume.work = vec![job];
        let result = v.validate(&resume).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("was responsible for")),
            "Expected passive voice warning for 'was responsible for', got: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_passive_voice_reports_each_phrase_separately() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        let mut job = make_work("Acme Corp", "Engineer");
        // Two distinct passive-voice phrases in the same field
        job.summary =
            Some("Was responsible for code reviews. Assisted in deployment processes.".to_string());
        resume.work = vec![job];
        let result = v.validate(&resume).unwrap();
        let passive_warnings: Vec<_> = result
            .warnings
            .iter()
            .filter(|w| w.contains("active voice"))
            .collect();
        assert!(
            passive_warnings.len() >= 2,
            "Expected at least 2 separate passive voice warnings, got: {:?}",
            passive_warnings
        );
        assert!(
            passive_warnings
                .iter()
                .any(|w| w.contains("was responsible for")),
            "Expected warning containing 'was responsible for'"
        );
        assert!(
            passive_warnings.iter().any(|w| w.contains("assisted in")),
            "Expected warning containing 'assisted in'"
        );
    }

    // -----------------------------------------------------------------------
    // check_length_optimization — coverage
    // -----------------------------------------------------------------------

    #[test]
    fn test_length_too_short_warning() {
        // A minimal resume with only a short summary is well under 150 words
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        resume.basics.summary = Some("Short summary.".to_string());
        let result = v.validate(&resume).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("content appears very thin")),
            "Expected thin-content warning, got: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_length_too_long_warning() {
        let v = ResumeValidator::new();
        let mut resume = minimal_resume("Test User");
        // Construct a summary that pushes word count past 3000
        let long_summary = (0..3100)
            .map(|i| format!("word{}", i))
            .collect::<Vec<_>>()
            .join(" ");
        resume.basics.summary = Some(long_summary);
        let result = v.validate(&resume).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("consider trimming")),
            "Expected too-long warning, got: {:?}",
            result.warnings
        );
    }
}
