use clap::{Parser, ValueEnum};
use provlint_core::config::{Format, LintConfig};
use provlint_core::diagnostic::Severity;
use provlint_core::ProvLint;
use std::path::PathBuf;
use std::process;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "provlint", version, about = "Lint Kickstart, AutoYaST, and Autoinstall files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Lint one or more files
    Lint {
        /// Files or directories to lint
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Force a specific format (auto-detected if omitted)
        #[arg(short, long)]
        format: Option<FormatArg>,

        /// Output format
        #[arg(short, long, default_value = "text")]
        output: OutputFormat,

        /// Recurse into directories
        #[arg(short, long)]
        recursive: bool,

        /// Disable specific rules (comma-separated)
        #[arg(long, value_delimiter = ',')]
        disable: Vec<String>,
    },
    /// List all supported rules
    Rules,
}

#[derive(Clone, ValueEnum)]
enum FormatArg {
    Kickstart,
    Autoyast,
    Autoinstall,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

impl From<FormatArg> for Format {
    fn from(f: FormatArg) -> Format {
        match f {
            FormatArg::Kickstart => Format::Kickstart,
            FormatArg::Autoyast => Format::AutoYaST,
            FormatArg::Autoinstall => Format::Autoinstall,
        }
    }
}

fn collect_files(paths: &[PathBuf], recursive: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            files.push(path.clone());
        } else if path.is_dir() && recursive {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let ext = entry
                        .path()
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    if matches!(ext, "ks" | "cfg" | "xml" | "yaml" | "yml" | "j2") {
                        files.push(entry.path().to_path_buf());
                    }
                }
            }
        } else if path.is_dir() {
            eprintln!(
                "Warning: '{}' is a directory; use --recursive to lint recursively",
                path.display()
            );
        } else {
            eprintln!("Warning: '{}' not found", path.display());
        }
    }
    files
}

fn main() {
    let cli = Cli::parse();
    let linter = ProvLint::new();

    match cli.command {
        Commands::Lint {
            paths,
            format,
            output,
            recursive,
            disable,
        } => {
            let files = collect_files(&paths, recursive);

            if files.is_empty() {
                eprintln!("No files found to lint");
                process::exit(1);
            }

            let config = LintConfig {
                disabled_rules: disable,
            };

            let mut total_errors = 0;
            let mut total_warnings = 0;
            let mut all_json_results: Vec<serde_json::Value> = Vec::new();

            for file in &files {
                let content = match std::fs::read_to_string(file) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error reading {}: {}", file.display(), e);
                        continue;
                    }
                };

                let fmt = format
                    .as_ref()
                    .map(|f| Format::from(f.clone()))
                    .or_else(|| {
                        file.extension()
                            .and_then(|e| e.to_str())
                            .and_then(Format::from_extension)
                    })
                    .or_else(|| Format::detect(&content));

                let fmt = match fmt {
                    Some(f) => f,
                    None => {
                        eprintln!(
                            "Warning: Could not detect format for '{}'; skipping",
                            file.display()
                        );
                        continue;
                    }
                };

                let diagnostics = linter.lint_with_config(&content, fmt, &config);

                match output {
                    OutputFormat::Text => {
                        if !diagnostics.is_empty() {
                            println!("\n{}", file.display());
                            println!("{}", "-".repeat(file.display().to_string().len()));
                        }
                        for diag in &diagnostics {
                            let severity_icon = match diag.severity {
                                Severity::Error => "error",
                                Severity::Warning => "warning",
                                Severity::Info => "info",
                            };
                            println!(
                                "  {}:{} {} [{}] {}",
                                diag.span.start_line,
                                diag.span.start_col,
                                severity_icon,
                                diag.code,
                                diag.message,
                            );
                            if let Some(ref fix) = diag.fix {
                                println!("    fix: {}", fix.description);
                            }
                        }
                    }
                    OutputFormat::Json => {
                        let result = serde_json::json!({
                            "file": file.display().to_string(),
                            "format": format!("{:?}", fmt).to_lowercase(),
                            "diagnostics": diagnostics,
                        });
                        all_json_results.push(result);
                    }
                }

                for diag in &diagnostics {
                    match diag.severity {
                        Severity::Error => total_errors += 1,
                        Severity::Warning => total_warnings += 1,
                        Severity::Info => {}
                    }
                }
            }

            if matches!(output, OutputFormat::Json) {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&all_json_results).unwrap()
                );
            } else {
                println!(
                    "\n{} file(s) linted: {} error(s), {} warning(s)",
                    files.len(),
                    total_errors,
                    total_warnings,
                );
            }

            if total_errors > 0 {
                process::exit(1);
            }
        }
        Commands::Rules => {
            let rules = linter.supported_rules();
            println!("{:<10} {:<60} Formats", "Code", "Description");
            println!("{}", "-".repeat(90));
            for rule in rules {
                let formats: Vec<&str> = rule
                    .formats
                    .iter()
                    .map(|f| match f {
                        Format::Kickstart => "kickstart",
                        Format::AutoYaST => "autoyast",
                        Format::Autoinstall => "autoinstall",
                    })
                    .collect();
                println!(
                    "{:<10} {:<60} {}",
                    rule.code,
                    rule.description,
                    formats.join(", ")
                );
            }
        }
    }
}
