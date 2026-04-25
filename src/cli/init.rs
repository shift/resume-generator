//! `init` subcommand handler.

use anyhow::Result;
use tracing::info;

pub fn run(name: Option<String>) -> Result<()> {
    let project_name = name.unwrap_or_else(|| "my-resume".to_string());

    info!("Initializing resume project: {}", project_name);

    let project_dir = std::path::Path::new(&project_name);

    if project_dir.exists() {
        return Err(anyhow::anyhow!(
            "Directory '{}' already exists. Remove it or choose a different name.",
            project_name
        ));
    }

    std::fs::create_dir_all(project_dir.join("templates/latex"))?;
    std::fs::create_dir_all(project_dir.join("templates/html"))?;
    std::fs::create_dir_all(project_dir.join("templates/text"))?;
    std::fs::create_dir_all(project_dir.join("build"))?;
    std::fs::create_dir_all(project_dir.join("tests"))?;

    let example_resume = r#"{
  "$schema": "https://raw.githubusercontent.com/jsonresume/resume-schema/v1.0.0/schema.json",
  "basics": {
    "name": "Your Name",
    "label": "Your Professional Title",
    "email": "your.email@example.com",
    "phone": "+1 (555) 123-4567",
    "summary": "A brief professional summary highlighting your key qualifications and experience."
  },
  "work": [
    {
      "name": "Company Name",
      "position": "Your Position",
      "startDate": "2020",
      "endDate": "Present",
      "summary": "Brief description of your role and responsibilities.",
      "highlights": [
          "Key achievement or responsibility 1",
          "Key achievement or responsibility 2"
      ]
    }
  ],
  "education": [
    {
      "institution": "University Name",
      "area": "Field of Study",
      "studyType": "Degree Type",
      "startDate": "2015",
      "endDate": "2019"
    }
  ],
  "skills": [
    {
      "name": "Category Name",
      "keywords": ["Skill 1", "Skill 2", "Skill 3"]
    }
  ]
}"#;

    std::fs::write(project_dir.join("resume.json"), example_resume)?;

    println!("Initialized resume project at ./{}/", project_name);

    Ok(())
}
