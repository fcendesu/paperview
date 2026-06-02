use std::{
    fmt, fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexCompileInput {
    entry_path: PathBuf,
    output_path: PathBuf,
    compiler_path: PathBuf,
}

impl TexCompileInput {
    #[must_use]
    pub fn new(entry_path: impl Into<PathBuf>) -> Self {
        let entry_path = entry_path.into();
        let output_path = tex_pdf_artifact_path(&entry_path);

        Self {
            entry_path,
            output_path,
            compiler_path: PathBuf::from("tectonic"),
        }
    }

    #[must_use]
    pub fn with_output_path(mut self, output_path: impl Into<PathBuf>) -> Self {
        self.output_path = output_path.into();
        self
    }

    #[must_use]
    pub fn with_compiler_path(mut self, compiler_path: impl Into<PathBuf>) -> Self {
        self.compiler_path = compiler_path.into();
        self
    }

    #[must_use]
    pub fn entry_path(&self) -> &Path {
        &self.entry_path
    }

    #[must_use]
    pub fn output_path(&self) -> &Path {
        &self.output_path
    }

    #[must_use]
    pub fn compiler_path(&self) -> &Path {
        &self.compiler_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexCompileArtifact {
    output_path: PathBuf,
    diagnostics: String,
}

impl TexCompileArtifact {
    #[must_use]
    pub fn new(output_path: impl Into<PathBuf>, diagnostics: impl Into<String>) -> Self {
        Self {
            output_path: output_path.into(),
            diagnostics: diagnostics.into(),
        }
    }

    #[must_use]
    pub fn output_path(&self) -> &Path {
        &self.output_path
    }

    #[must_use]
    pub fn diagnostics(&self) -> &str {
        &self.diagnostics
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TexCompileError {
    CompilerUnavailable {
        compiler_path: PathBuf,
        source: String,
    },
    CompileFailed {
        path: PathBuf,
        source: String,
    },
    OutputMissing {
        path: PathBuf,
    },
    WriteFailed {
        path: PathBuf,
        source: String,
    },
}

impl fmt::Display for TexCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CompilerUnavailable {
                compiler_path,
                source,
            } => {
                write!(
                    formatter,
                    "failed to run Tectonic compiler {}: {source}",
                    compiler_path.display()
                )
            }
            Self::CompileFailed { path, source } => write!(
                formatter,
                "failed to compile {} with Tectonic: {source}",
                path.display()
            ),
            Self::OutputMissing { path } => {
                write!(formatter, "Tectonic did not produce {}", path.display())
            }
            Self::WriteFailed { path, source } => {
                write!(formatter, "failed to write {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for TexCompileError {}

pub fn compile_tex(input: &TexCompileInput) -> Result<TexCompileArtifact, TexCompileError> {
    let output_dir = input
        .output_path()
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), PathBuf::from);

    fs::create_dir_all(&output_dir).map_err(|error| TexCompileError::WriteFailed {
        path: output_dir.clone(),
        source: error.to_string(),
    })?;

    let output = Command::new(input.compiler_path())
        .arg("--outdir")
        .arg(&output_dir)
        .arg(input.entry_path())
        .output()
        .map_err(|error| TexCompileError::CompilerUnavailable {
            compiler_path: input.compiler_path.clone(),
            source: error.to_string(),
        })?;

    let diagnostics = tectonic_diagnostics(&output);
    if !output.status.success() {
        return Err(TexCompileError::CompileFailed {
            path: input.entry_path.clone(),
            source: diagnostics,
        });
    }

    let generated_output_path = output_dir.join(tectonic_generated_pdf_name(input.entry_path()));
    if !generated_output_path.exists() {
        return Err(TexCompileError::OutputMissing {
            path: generated_output_path,
        });
    }

    if generated_output_path != input.output_path() {
        fs::rename(&generated_output_path, input.output_path())
            .or_else(|_| {
                fs::copy(&generated_output_path, input.output_path()).map(|_| {
                    let _ = fs::remove_file(&generated_output_path);
                })
            })
            .map_err(|error| TexCompileError::WriteFailed {
                path: input.output_path.clone(),
                source: error.to_string(),
            })?;
    }

    Ok(TexCompileArtifact::new(
        input.output_path.clone(),
        diagnostics,
    ))
}

#[must_use]
pub fn tex_pdf_artifact_path(entry_path: impl AsRef<Path>) -> PathBuf {
    let entry_path = entry_path.as_ref();
    let file_name = entry_path.file_name().unwrap_or(entry_path.as_os_str());
    let artifact_name = Path::new(file_name).with_extension("pdf");

    entry_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map_or_else(
            || PathBuf::from(".paperview").join("tex").join(&artifact_name),
            |parent| parent.join(".paperview").join("tex").join(&artifact_name),
        )
}

fn tectonic_generated_pdf_name(entry_path: &Path) -> PathBuf {
    entry_path.file_name().map_or_else(
        || entry_path.with_extension("pdf"),
        |file_name| Path::new(file_name).with_extension("pdf"),
    )
}

fn tectonic_diagnostics(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let diagnostics = [stdout.trim(), stderr.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    if diagnostics.is_empty() {
        "compiled with Tectonic".to_owned()
    } else {
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{TexCompileError, TexCompileInput, compile_tex, tex_pdf_artifact_path};

    #[test]
    fn tex_pdf_artifact_path_replaces_extension() {
        assert_eq!(
            tex_pdf_artifact_path("resume.tex"),
            PathBuf::from(".paperview/tex/resume.pdf")
        );
        assert_eq!(
            tex_pdf_artifact_path("archive.resume.tex"),
            PathBuf::from(".paperview/tex/archive.resume.pdf")
        );
        assert_eq!(
            tex_pdf_artifact_path("docs/resume.tex"),
            PathBuf::from("docs/.paperview/tex/resume.pdf")
        );
    }

    #[test]
    fn tex_compile_input_defaults_to_managed_artifact_path() {
        let input = TexCompileInput::new("docs/resume.tex");

        assert_eq!(input.entry_path(), PathBuf::from("docs/resume.tex"));
        assert_eq!(
            input.output_path(),
            PathBuf::from("docs/.paperview/tex/resume.pdf")
        );
        assert_eq!(input.compiler_path(), PathBuf::from("tectonic"));
    }

    #[test]
    fn tex_compile_input_accepts_explicit_output_path() {
        let input =
            TexCompileInput::new("resume.tex").with_output_path("target/paperview/resume.pdf");

        assert_eq!(input.entry_path(), PathBuf::from("resume.tex"));
        assert_eq!(
            input.output_path(),
            PathBuf::from("target/paperview/resume.pdf")
        );
    }

    #[test]
    fn compile_tex_reports_missing_compiler() {
        let input = TexCompileInput::new("missing-resume.tex")
            .with_compiler_path("/definitely-missing-paperview-tectonic");
        let error = compile_tex(&input).expect_err("compile should fail");

        assert!(matches!(error, TexCompileError::CompilerUnavailable { .. }));
        assert!(
            error
                .to_string()
                .contains("/definitely-missing-paperview-tectonic")
        );
    }

    #[test]
    fn compile_tex_writes_pdf_artifact() {
        let stem = unique_test_stem("minimal-tex");
        let tex_path = std::env::temp_dir().join(format!("{stem}.tex"));
        let pdf_path = std::env::temp_dir().join(format!("{stem}-custom.pdf"));
        let compiler_path = fake_tectonic_compiler(&stem, true);
        fs::write(
            &tex_path,
            r"\documentclass{article}
\begin{document}
Hello from PaperView.
\end{document}
",
        )
        .expect("write tex fixture");

        let input = TexCompileInput::new(&tex_path)
            .with_output_path(&pdf_path)
            .with_compiler_path(&compiler_path);
        let artifact = compile_tex(&input).expect("compile tex fixture");

        assert_eq!(artifact.output_path(), pdf_path.as_path());
        assert!(artifact.diagnostics().contains("fake tectonic ok"));
        assert!(fs::read(&pdf_path).expect("read pdf").starts_with(b"%PDF"));

        fs::remove_file(tex_path).expect("remove tex fixture");
        fs::remove_file(pdf_path).expect("remove pdf fixture");
        fs::remove_file(compiler_path).expect("remove fake compiler");
    }

    #[test]
    fn compile_tex_reports_compiler_failure() {
        let stem = unique_test_stem("failed-tex");
        let tex_path = std::env::temp_dir().join(format!("{stem}.tex"));
        let compiler_path = fake_tectonic_compiler(&stem, false);
        fs::write(&tex_path, "\\documentclass{article}").expect("write tex fixture");

        let input = TexCompileInput::new(&tex_path).with_compiler_path(&compiler_path);
        let error = compile_tex(&input).expect_err("compile should fail");

        assert!(matches!(error, TexCompileError::CompileFailed { .. }));
        assert!(error.to_string().contains("fake tectonic failed"));

        fs::remove_file(tex_path).expect("remove tex fixture");
        fs::remove_file(compiler_path).expect("remove fake compiler");
    }

    #[test]
    fn compile_tex_reports_missing_output() {
        let stem = unique_test_stem("missing-output-tex");
        let tex_path = std::env::temp_dir().join(format!("{stem}.tex"));
        let compiler_path = fake_tectonic_compiler_without_output(&stem);
        fs::write(&tex_path, "\\documentclass{article}").expect("write tex fixture");

        let input = TexCompileInput::new(&tex_path).with_compiler_path(&compiler_path);
        let error = compile_tex(&input).expect_err("compile should fail");

        assert!(matches!(error, TexCompileError::OutputMissing { .. }));

        fs::remove_file(tex_path).expect("remove tex fixture");
        fs::remove_file(compiler_path).expect("remove fake compiler");
    }

    fn unique_test_stem(prefix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        format!("{prefix}-{nanos}")
    }

    #[cfg(unix)]
    fn fake_tectonic_compiler(stem: &str, succeeds: bool) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let compiler_path = std::env::temp_dir().join(format!("{stem}-tectonic"));
        let script = if succeeds {
            r#"#!/bin/sh
outdir="."
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--outdir" ]; then
    shift
    outdir="$1"
  else
    input="$1"
  fi
  shift
done
stem=$(basename "$input" .tex)
printf '%s\n' '%PDF fake' > "$outdir/$stem.pdf"
printf '%s\n' 'fake tectonic ok'
"#
        } else {
            r#"#!/bin/sh
printf '%s\n' 'fake tectonic failed' >&2
exit 1
"#
        };

        fs::write(&compiler_path, script).expect("write fake compiler");
        let mut permissions = fs::metadata(&compiler_path)
            .expect("fake compiler metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&compiler_path, permissions).expect("set fake compiler executable");
        compiler_path
    }

    #[cfg(unix)]
    fn fake_tectonic_compiler_without_output(stem: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let compiler_path = std::env::temp_dir().join(format!("{stem}-tectonic"));
        fs::write(
            &compiler_path,
            r#"#!/bin/sh
printf '%s\n' 'fake tectonic ok without output'
"#,
        )
        .expect("write fake compiler");
        let mut permissions = fs::metadata(&compiler_path)
            .expect("fake compiler metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&compiler_path, permissions).expect("set fake compiler executable");
        compiler_path
    }

    #[cfg(not(unix))]
    fn fake_tectonic_compiler(_stem: &str, _succeeds: bool) -> PathBuf {
        panic!("fake compiler tests require a Unix shell")
    }

    #[cfg(not(unix))]
    fn fake_tectonic_compiler_without_output(_stem: &str) -> PathBuf {
        panic!("fake compiler tests require a Unix shell")
    }
}
