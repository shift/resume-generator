{
  description = "JSON Resume Builder - Fast, safe resume generation with Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # Rust toolchain for development
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # Original LaTeX environment for backward compatibility
        tex = pkgs.texlive.combine {
          inherit (pkgs.texlive) scheme-basic
            latex-bin
            latexmk
            moderncv
            fontawesome5
            multirow
            arydshln
            xcolor
            etoolbox
            moderntimeline
            fancyhdr
            microtype
            pgf
            geometry
            hyperref
            l3packages
            ec
            cm-super;
        };

        # Additional tools for ResumeOps
        resumeTools = with pkgs; [
          tectonic  # Fast LaTeX compilation
          poppler_utils  # pdftotext for ATS simulation
          diff-pdf  # Visual regression testing
          vale  # Content linting
          nodePackages.alex  # Bias language detection
          git  # Version control
          ripgrep  # Fast search
        ];

        # Python environment for additional tools (optional)
        pythonEnv = pkgs.python311.withPackages (ps: with ps; [
          jsonschema  # JSON Resume validation
          pandas  # Data analysis for ATS optimization
        ]);
      in
      {
        # Original LaTeX build (backward compatibility)
        packages.latex-pdf = pkgs.stdenvNoCC.mkDerivation {
          name = "cv-latex";
          src = ./.;
          buildInputs = [ tex ];
          buildPhase = ''
            latexmk -pdf cv.tex
          '';
          installPhase = ''
            mkdir -p $out
            cp cv.pdf $out/
          '';
        };

        # New Rust-based JSON Resume builder
        packages.resume-builder = pkgs.rustPlatform.buildRustPackage {
          pname = "resume-builder";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildInputs = [ tex ];  # For LaTeX compilation
        };

        # Multi-format build target
        packages.all-formats = pkgs.stdenvNoCC.mkDerivation {
          name = "resume-all-formats";
          src = ./.;
          buildInputs = [ 
            rustToolchain 
            tex
            pythonEnv
            resumeTools
          ];
          buildPhase = ''
            # Build Rust binary first
            cargo build --release
            
            # Create output directory
            mkdir -p $out
            
            # Generate all formats
            ./target/release/resume-builder build \
              --resume resume.json \
              --format all \
              --output $out
          '';
          installPhase = ''
            echo "All resume formats built in $out"
          '';
        };

        # Default package
        packages.default = self.packages.${system}.resume-builder;

        # Enhanced development shell
        devShells.default = pkgs.mkShell {
          buildInputs = [
            # Rust development
            rustToolchain
            pkgs.rust-analyzer
            pkgs.cargo-watch
            
            # LaTeX environment
            tex
            
            # ResumeOps tools
            resumeTools
            pythonEnv
            
            # Development utilities
            pkgs.pre-commit
            pkgs.treefmt
            pkgs.nodePackages.prettier
          ];
          
          shellHook = ''
            # Enable Rust development environment
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
            
            # Enable LaTeX tools
            export PATH="${tex}/bin:$PATH"
            
            # ResumeOps aliases
            alias resume="./target/release/resume-builder"
            alias resume-dev="cargo run --"
            
            echo "🚀 JSON Resume Builder Development Environment Ready!"
            echo "Available commands:"
            echo "  resume-dev validate resume.json   # Validate resume"
            echo "  resume-dev build --format pdf    # Build PDF"
            echo "  resume-dev build --format html    # Build HTML"
            echo "  resume-dev build --format all     # Build all formats"
            echo "  resume-dev keywords resume.json  # Extract ATS keywords"
            echo ""
            echo "Legacy LaTeX build still available:"
            echo "  latexmk -pdf cv.tex"
          '';
        };

        # Testing shell with additional tools
        devShells.testing = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            tex
            resumeTools
            pythonEnv
            
            # Testing specific tools
            pkgs.cargo-nextest
            pkgs.criterion-rs
            pkgs.cargo-audit
            pkgs.cargo-deny
          ];
          
          shellHook = ''
            echo "🧪 JSON Resume Testing Environment Ready!"
            echo "Available commands:"
            echo "  cargo nextest run             # Fast test runner"
            echo "  cargo test                    # Standard tests"
            echo "  cargo bench                   # Performance tests"
            echo "  cargo audit                   # Security audit"
            echo "  cargo deny check              # License/compliance check"
          '';
        };
      }
    );
}
