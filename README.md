# JSON Resume Builder

A fast, safe JSON Resume builder written in Rust. Generates professional PDF, HTML, and ATS-friendly text output from a single [JSON Resume](https://jsonresume.org/schema/) data file, with built-in validation and content quality checking.

## Features

- **Multi-format output** — LaTeX/PDF, HTML (4 themes), and plain-text ATS format from one source file
- **Validation** — schema compliance, date ordering, email format, action verb detection, weasel word and passive voice checks
- **ATS optimisation** — keyword extraction, quantification ratio analysis, plain-text output with ASCII-safe formatting
- **Template engine** — Tera (Jinja2-compatible) templates; bring your own or use the bundled ones
- **LaTeX auto-detection** — probes for Tectonic, XeLaTeX, and latexmk; uses the first one found
- **Safe defaults** — `-shell-escape` off by default, aux file cleanup after PDF compilation
- **Nix-native** — reproducible builds via Nix flakes

## Quick Start

```bash
# With Nix
nix develop
cargo build --release

# Without Nix (requires Rust toolchain)
cargo build --release
```

```bash
# Initialise a new project
./target/release/resume-builder init --name my-resume

# Edit my-resume/resume.json, then:

# Validate
./target/release/resume-builder validate --resume my-resume/resume.json

# Build all formats
./target/release/resume-builder build \
  --resume my-resume/resume.json \
  --format all \
  --output my-resume/build/
```

## Commands

### `validate`

Check a resume for schema errors, content quality issues, and ATS warnings.

```bash
resume-builder validate --resume resume.json
```

Exits `0` if valid (warnings are non-fatal). Exits `1` if there are hard errors.

**What it checks:**
- `basics.name`, `basics.email` (regex), `basics.phone` (digit count)
- Work entry required fields, date ordering (month-granular), summary length, action verbs in highlights
- Education required fields and date ordering
- Duplicate skills, empty or very short keywords
- Project name and description length
- ATS keyword count and quantification ratio
- Weasel words (`highly`, `very`, `excellent`, etc.) and passive voice across all text content

---

### `build`

Generate resume output in one or more formats.

```bash
resume-builder build [OPTIONS]
```

| Flag | Short | Default | Description |
|---|---|---|---|
| `--resume` | `-r` | `resume.json` | Path to JSON Resume file |
| `--format` | `-f` | `pdf` | Output format: `latex`, `html`, `pdf`, `text`, `all` |
| `--output` | `-o` | `build/` | Output directory |
| `--template-dir` | | `templates/` | Directory containing Tera templates |
| `--theme` | `-t` | `modern` | HTML theme (see below) |

**Examples:**

```bash
# PDF only (auto-detects LaTeX engine)
resume-builder build --resume resume.json --format pdf

# HTML with a specific theme
resume-builder build --resume resume.json --format html --theme tech-timeline

# All formats to a custom output directory
resume-builder build --resume resume.json --format all --output dist/

# Use custom templates
resume-builder build --resume resume.json --template-dir /path/to/my-templates/
```

---

### `keywords`

Extract and analyse keywords from the resume for ATS optimisation.

```bash
resume-builder keywords --resume resume.json
```

Prints a ranked list of all extracted keywords with occurrence counts.

---

### `init`

Scaffold a new resume project directory.

```bash
resume-builder init --name my-resume
```

Creates `my-resume/` with subdirectories `templates/`, `build/`, `tests/` and a starter `resume.json`. Errors if the directory already exists.

---

## HTML Themes

Four themes are available via `--theme`:

| Theme | Flag value | Description |
|---|---|---|
| Modern (default) | `modern` | Clean two-column layout with skill bars |
| Clean Minimal | `clean-minimal` | Single-column editorial style |
| Modern Professional | `modern-professional` | Bold header, card-based sections |
| Tech Timeline | `tech-timeline` | Timeline-style work history |

```bash
resume-builder build --resume resume.json --format html --theme modern-professional
```

---

## JSON Resume Schema

This tool implements [JSON Resume v1.0.0](https://jsonresume.org/schema/). All top-level sections are optional — only include what you have.

```json
{
  "$schema": "https://raw.githubusercontent.com/jsonresume/resume-schema/v1.0.0/schema.json",
  "basics": {
    "name": "Your Name",
    "label": "Professional Title",
    "email": "you@example.com",
    "phone": "+44 7700 000000",
    "url": "https://yoursite.com",
    "summary": "Brief professional summary.",
    "location": {
      "city": "London",
      "countryCode": "GB"
    },
    "profiles": [
      { "network": "GitHub", "username": "yourhandle", "url": "https://github.com/yourhandle" },
      { "network": "LinkedIn", "username": "yourhandle", "url": "https://linkedin.com/in/yourhandle" }
    ]
  },
  "work": [
    {
      "name": "Company Name",
      "position": "Your Role",
      "url": "https://company.com",
      "startDate": "2021-03",
      "endDate": "Present",
      "summary": "What you did.",
      "highlights": [
        "Reduced deployment time by 40% by introducing a CI/CD pipeline.",
        "Led a team of 5 engineers across 3 time zones."
      ],
      "location": "London, UK"
    }
  ],
  "education": [
    {
      "institution": "University Name",
      "area": "Computer Science",
      "studyType": "BSc",
      "startDate": "2014",
      "endDate": "2018"
    }
  ],
  "skills": [
    { "name": "Languages", "keywords": ["Rust", "Go", "Python", "TypeScript"] },
    { "name": "Infrastructure", "keywords": ["Kubernetes", "Terraform", "AWS"] }
  ],
  "projects": [
    {
      "name": "my-oss-project",
      "description": "A tool that does X.",
      "url": "https://github.com/you/my-oss-project",
      "highlights": ["500+ GitHub stars", "Used in production at 3 companies"],
      "startDate": "2022"
    }
  ],
  "awards": [
    { "title": "Award Name", "date": "2023", "awarder": "Organisation", "summary": "Why." }
  ],
  "certificates": [
    { "name": "Certified Kubernetes Administrator", "date": "2022", "issuer": "CNCF" }
  ],
  "publications": [
    { "name": "Article Title", "publisher": "Blog Name", "releaseDate": "2023-06", "url": "https://..." }
  ],
  "languages": [
    { "language": "English", "fluency": "Native" },
    { "language": "German", "fluency": "Professional" }
  ],
  "volunteer": [
    {
      "organization": "Organisation Name",
      "position": "Role",
      "startDate": "2019",
      "endDate": "2021",
      "summary": "What you did."
    }
  ],
  "interests": [
    { "name": "Open Source", "keywords": ["Rust", "Nix", "Linux"] }
  ],
  "references": [
    { "name": "Referee Name", "reference": "Available on request." }
  ]
}
```

**Date format:** ISO 8601 — `YYYY`, `YYYY-MM`, or `YYYY-MM-DD`. Use `"Present"` for ongoing roles.

---

## Templates

Templates use [Tera](https://keats.github.io/tera/) syntax (Jinja2-compatible). Fields map to camelCase keys matching the JSON Resume spec.

### Bundled templates

```
templates/
├── latex/
│   └── moderncv.tex.j2        # ModernCV-based LaTeX
├── html/
│   ├── modern.html.j2          # Default HTML theme
│   └── themes/
│       ├── clean-minimal.html.j2
│       ├── modern-professional.html.j2
│       └── tech-timeline.html.j2
└── text/
    └── ats-friendly.txt.j2    # ATS plain-text
```

### Custom templates

Point `--template-dir` at any directory containing `.j2` files with the same naming conventions. The root context variable is `resume` — the full deserialized JSON Resume object.

```jinja2
{{ resume.basics.name }}
{{ resume.basics.label }}
{% for job in resume.work %}
  {{ job.position }} at {{ job.name }}
  {{ job.startDate }} - {{ job.endDate | default(value="Present") }}
{% endfor %}
```

The `escape_latex` filter is available in LaTeX templates to safely escape special characters:

```jinja2
{{ job.summary | escape_latex }}
```

---

## LaTeX / PDF Compilation

The `pdf` format requires a LaTeX engine. The builder probes for engines in this order:

1. **Tectonic** — self-contained, no TeX installation needed ([tectonic-typesetting.io](https://tectonic-typesetting.io))
2. **XeLaTeX** — from a standard TeX Live or MiKTeX installation
3. **latexmk** — wrapper around pdflatex/xelatex

If none are found, `build --format pdf` returns an error. Use `--format latex` to produce the `.tex` source without compiling.

Auxiliary files (`.aux`, `.log`, `.fls`, `.fdb_latexmk`, `.out`) are cleaned up automatically after a successful PDF build.

---

## Project Structure

```
.
├── resume.json                  # Your resume data
├── Cargo.toml
├── flake.nix                    # Nix build
├── src/
│   ├── main.rs                  # CLI entry point and argument definitions
│   ├── lib.rs                   # Library exports
│   ├── cli/                     # Subcommand handlers
│   │   ├── build.rs
│   │   ├── init.rs
│   │   ├── keywords.rs
│   │   └── validate.rs
│   ├── models/
│   │   └── resume.rs            # JSON Resume data structures
│   ├── templates/
│   │   └── engine.rs            # Tera engine wrapper and escape_latex filter
│   ├── utils/
│   │   └── latex.rs             # LaTeX engine detection and compilation
│   └── validation/
│       └── resume_validator.rs  # All validation and content quality logic
├── templates/                   # Tera template files
├── tests/
│   ├── cli_scenarios.rs         # End-to-end CLI tests (assert_cmd)
│   ├── template_snapshots.rs    # Snapshot tests (insta)
│   └── fixtures/                # Test resume JSON files
└── build/                       # Generated output (gitignored)
```

---

## Development

```bash
# Build
cargo build

# Run all tests (63 tests across 4 suites)
cargo test

# Lint
cargo clippy

# Update snapshot tests after intentional template changes
cargo insta accept
cargo test
```

### Test suites

| Suite | Location | Count | What it covers |
|---|---|---|---|
| Unit — validator | `src/validation/resume_validator.rs` | 45 | All validation rules, property tests |
| Unit — templates | `src/templates/engine.rs` | 4 | `escape_latex` filter, property tests |
| CLI scenarios | `tests/cli_scenarios.rs` | 5 | End-to-end binary behaviour |
| Snapshot | `tests/template_snapshots.rs` | 5 | Template rendering regression |

---

## License

MIT — see `LICENSE` for details.
