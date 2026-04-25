#!/usr/bin/env nix-shell
#!nix

{ pkgs ? import <nixpkgs> {} }:

with pkgs;

mkShell {
  buildInputs = [
    vale
    alex
    poppler_utils  # For pdftotext
    nodePackages_20_x.prettier  # For formatting
    fd  # Fast find
    ripgrep
  ];

  shellHook = ''
    echo "🔍 Resume Content Quality Tools Ready!"
    echo ""
    echo "Available commands:"
    echo "  vale-all              Check all generated formats"
    echo "  vale-resume           Check ATS text format"
    echo "  vale-html             Check HTML format"
    echo "  alex-all              Run Alex on all text formats"
    echo "  quality-report         Generate comprehensive quality report"
    echo ""
    
    # Custom aliases for content quality checking
    vale-all() {
      echo "🔍 Running Vale on all formats..."
      for file in dist/*.txt dist/*.html; do
        if [[ -f "$file" ]]; then
          echo "Checking $file..."
          vale "$file"
        fi
      done
    }
    
    vale-resume() {
      echo "📝 Checking ATS text resume..."
      vale dist/resume.txt
    }
    
    vale-html() {
      echo "🌐 Checking HTML resume..."
      vale dist/resume.html
    }
    
    alex-all() {
      echo "🚫 Running Alex (bias detection) on all text formats..."
      for file in dist/*.txt; do
        if [[ -f "$file" ]]; then
          echo "Checking $file for biased language..."
          alex --quiet "$file"
        fi
      done
    }
    
    quality-report() {
      echo "📊 Generating comprehensive quality report..."
      echo "Resume Content Quality Report" > dist/quality-report.md
      echo "=============================" >> dist/quality-report.md
      echo "" >> dist/quality-report.md
      
      echo "## 📝 Vale Content Analysis" >> dist/quality-report.md
      if [[ -f dist/resume.txt ]]; then
        echo "### ATS Text Analysis:" >> dist/quality-report.md
        echo '```' >> dist/quality-report.md
        vale --output=line dist/resume.txt >> dist/quality-report.md 2>&1 || true
        echo '```' >> dist/quality-report.md
        echo "" >> dist/quality-report.md
      fi
      
      echo "## 🚫 Alex Bias Detection" >> dist/quality-report.md
      if [[ -f dist/resume.txt ]]; then
        echo "### ATS Text Bias Check:" >> dist/quality-report.md
        echo '```' >> dist/quality-report.md
        alex --quiet dist/resume.txt >> dist/quality-report.md 2>&1 || echo "No biased language detected" >> dist/quality-report.md
        echo '```' >> dist/quality-report.md
        echo "" >> dist/quality-report.md
      fi
      
      echo "## 📈 Keyword Density Analysis" >> dist/quality-report.md
      if [[ -f dist/resume.txt ]]; then
        echo "### Top Keywords by Frequency:" >> dist/quality-report.md
        echo '```' >> dist/quality-report.md
        rg --count-sort --no-filename --ignore-case -o '[a-z]{3,}' dist/resume.txt | head -20 >> dist/quality-report.md
        echo '```' >> dist/quality-report.md
        echo "" >> dist/quality-report.md
      fi
      
      echo "## 📏 Content Metrics" >> dist/quality-report.md
      if [[ -f dist/resume.txt ]]; then
        echo "- ATS Text Character Count: $(wc -c < dist/resume.txt)" >> dist/quality-report.md
        echo "- ATS Text Word Count: $(wc -w < dist/resume.txt)" >> dist/quality-report.md
        echo "- ATS Text Line Count: $(wc -l < dist/resume.txt)" >> dist/quality-report.md
        echo "" >> dist/quality-report.md
      fi
      
      echo "✅ Quality report generated: dist/quality-report.md"
    }
    
    export PATH="${vale}/bin:${alex}/bin:${poppler_utils}/bin:${nodePackages_20_x.prettier}/bin:${fd}/bin:${ripgrep}/bin:$PATH"
    
    echo "🎯 Environment ready for resume content quality analysis!"
    echo "Run 'quality-report' for a comprehensive analysis."
  '';
}