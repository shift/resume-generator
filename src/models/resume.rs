use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// JSON Resume structure following https://jsonresume.org/schema/
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Resume {
    #[serde(rename = "$schema", default)]
    pub schema: String,
    #[serde(default)]
    pub basics: Basics,
    #[serde(default)]
    pub work: Vec<Work>,
    #[serde(default)]
    pub volunteer: Vec<Volunteer>,
    #[serde(default)]
    pub education: Vec<Education>,
    #[serde(default)]
    pub awards: Vec<Award>,
    #[serde(default)]
    pub certificates: Vec<Certificate>,
    #[serde(default)]
    pub publications: Vec<Publication>,
    #[serde(default)]
    pub skills: Vec<Skill>,
    #[serde(default)]
    pub languages: Vec<Language>,
    #[serde(default)]
    pub interests: Vec<Interest>,
    #[serde(default)]
    pub references: Vec<Reference>,
    #[serde(default)]
    pub projects: Vec<Project>,
    pub meta: Option<Meta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Basics {
    #[serde(default)]
    pub name: String,
    pub label: Option<String>,
    pub image: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub url: Option<String>,
    pub summary: Option<String>,
    pub location: Option<Location>,
    #[serde(default)]
    pub profiles: Vec<Profile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub address: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub network: String,
    pub username: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    pub name: String,
    pub position: String,
    pub url: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub highlights: Vec<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Volunteer {
    pub organization: String,
    pub position: String,
    pub url: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Education {
    pub institution: String,
    pub url: Option<String>,
    pub area: Option<String>,
    pub study_type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub score: Option<String>,
    #[serde(default)]
    pub courses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Award {
    pub title: String,
    pub date: Option<String>,
    pub awarder: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Certificate {
    pub name: String,
    pub date: Option<String>,
    pub url: Option<String>,
    pub issuer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Publication {
    pub name: String,
    pub publisher: Option<String>,
    pub release_date: Option<String>,
    pub url: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Skill {
    pub name: String,
    pub level: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Language {
    pub language: String,
    pub fluency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Interest {
    pub name: String,
    #[serde(default)]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Reference {
    pub name: String,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub highlights: Vec<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    pub entity: Option<String>,
    #[serde(rename = "type")]
    pub project_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub canonical: Option<String>,
    pub version: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    #[serde(skip)]
    #[allow(dead_code)]
    pub ai_optimization: Option<AiOptimization>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AiOptimization {
    pub strategic_assessment: Option<String>,
    pub hiring_signal: Option<String>,
    pub technical_assessment: Option<String>,
    pub leadership_assessment: Option<String>,
}

impl Resume {
    /// Load a resume from a JSON file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let resume: Resume = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse resume JSON: {}", e))?;
        Ok(resume)
    }

    /// Validate resume against JSON Resume schema
    #[allow(dead_code)]
    pub fn validate(&self) -> anyhow::Result<()> {
        // Basic validation - can be extended with full JSON schema validation
        if self.basics.name.is_empty() {
            return Err(anyhow::anyhow!("Resume basics.name is required"));
        }

        for (i, job) in self.work.iter().enumerate() {
            if job.name.is_empty() {
                return Err(anyhow::anyhow!("Work[{}].name is required", i));
            }
            if job.position.is_empty() {
                return Err(anyhow::anyhow!("Work[{}].position is required", i));
            }
        }

        Ok(())
    }

    /// Get all technical keywords for ATS optimization
    pub fn get_all_keywords(&self) -> Vec<String> {
        let mut keywords = Vec::new();

        for skill in &self.skills {
            keywords.extend(skill.keywords.iter().cloned());
            keywords.push(skill.name.clone());
        }

        for job in &self.work {
            if let Some(summary) = &job.summary {
                keywords.extend(extract_keywords_from_text(summary));
            }
            for highlight in &job.highlights {
                keywords.extend(extract_keywords_from_text(highlight));
            }
        }

        // Remove duplicates and return
        keywords.sort();
        keywords.dedup();
        keywords
    }
}

/// Simple keyword extraction (can be enhanced with NLP libraries)
fn extract_keywords_from_text(text: &str) -> Vec<String> {
    // This is a simple implementation - can be enhanced with proper NLP
    text.split_whitespace()
        .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|s| !s.is_empty() && s.len() > 2)
        .map(|s| s.to_lowercase())
        .collect()
}
