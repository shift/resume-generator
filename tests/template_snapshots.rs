use jsonresume_builder::models::resume::{Basics, Project, Resume, Skill, Work};
use jsonresume_builder::templates::engine::TemplateEngine;
use std::path::PathBuf;

fn build_test_resume() -> Resume {
    Resume {
        schema: "https://raw.githubusercontent.com/jsonresume/resume-schema/v1.0.0/schema.json"
            .to_string(),
        basics: Basics {
            name: "Test User".to_string(),
            label: None,
            image: None,
            email: Some("test@example.com".to_string()),
            phone: None,
            url: None,
            summary: Some("Test summary for snapshot.".to_string()),
            location: None,
            profiles: vec![],
        },
        work: vec![Work {
            name: "Test Corp".to_string(),
            position: "Engineer".to_string(),
            url: None,
            start_date: Some("2020".to_string()),
            end_date: None,
            summary: None,
            highlights: vec!["Led a team of 5.".to_string()],
            location: None,
        }],
        volunteer: vec![],
        education: vec![],
        awards: vec![],
        certificates: vec![],
        publications: vec![],
        skills: vec![Skill {
            name: "Testing".to_string(),
            level: None,
            keywords: vec!["Rust".to_string(), "proptest".to_string()],
        }],
        languages: vec![],
        interests: vec![],
        references: vec![],
        projects: vec![Project {
            name: "test-project".to_string(),
            description: Some("A test project.".to_string()),
            highlights: vec![],
            start_date: None,
            end_date: None,
            url: None,
            roles: vec![],
            entity: None,
            project_type: None,
        }],
        meta: None,
    }
}

fn make_engine() -> TemplateEngine {
    let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    TemplateEngine::new(&template_dir).expect("Failed to create TemplateEngine")
}

#[test]
fn snapshot_latex_contains_documentclass_and_name() {
    let resume = build_test_resume();
    let engine = make_engine();
    let latex = engine
        .render_latex(&resume)
        .expect("Failed to render LaTeX");
    assert!(
        latex.contains("\\documentclass"),
        "LaTeX output missing \\documentclass"
    );
    assert!(
        latex.contains("Test User"),
        "LaTeX output missing 'Test User'"
    );
    insta::assert_snapshot!("latex_output", latex);
}

#[test]
fn snapshot_html_contains_html_tag_and_name() {
    let resume = build_test_resume();
    let engine = make_engine();
    let html = engine
        .render_html(&resume, "modern")
        .expect("Failed to render HTML");
    assert!(html.contains("<html"), "HTML output missing '<html'");
    assert!(
        html.contains("Test User"),
        "HTML output missing 'Test User'"
    );
    insta::assert_snapshot!("html_output", html);
}

#[test]
fn snapshot_ats_text_contains_name() {
    let resume = build_test_resume();
    let engine = make_engine();
    let text = engine
        .render_ats_text(&resume)
        .expect("Failed to render ATS text");
    assert!(
        text.contains("Test User"),
        "ATS text output missing 'Test User'"
    );
    insta::assert_snapshot!("ats_text_output", text);
}

#[test]
fn snapshot_latex_highlight_flows_through() {
    let resume = build_test_resume();
    let engine = make_engine();
    let latex = engine
        .render_latex(&resume)
        .expect("Failed to render LaTeX");
    assert!(
        latex.contains("Led a team of 5."),
        "LaTeX output missing highlight 'Led a team of 5.'"
    );
}

#[test]
fn snapshot_html_highlight_flows_through() {
    let resume = build_test_resume();
    let engine = make_engine();
    let html = engine
        .render_html(&resume, "modern")
        .expect("Failed to render HTML");
    assert!(
        html.contains("Led a team of 5."),
        "HTML output missing highlight 'Led a team of 5.'"
    );
}
