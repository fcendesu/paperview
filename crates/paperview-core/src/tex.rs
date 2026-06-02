use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexCompileInput {
    entry_path: PathBuf,
    output_path: PathBuf,
}

impl TexCompileInput {
    #[must_use]
    pub fn new(entry_path: impl Into<PathBuf>) -> Self {
        let entry_path = entry_path.into();
        let output_path = tex_pdf_artifact_path(&entry_path);

        Self {
            entry_path,
            output_path,
        }
    }

    #[must_use]
    pub fn with_output_path(mut self, output_path: impl Into<PathBuf>) -> Self {
        self.output_path = output_path.into();
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
    UnsupportedUntilTectonicAdapter { entry_path: PathBuf },
}

impl fmt::Display for TexCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedUntilTectonicAdapter { entry_path } => write!(
                formatter,
                "Tectonic .tex compilation is not implemented yet: {}",
                entry_path.display()
            ),
        }
    }
}

impl std::error::Error for TexCompileError {}

pub fn compile_tex(input: &TexCompileInput) -> Result<TexCompileArtifact, TexCompileError> {
    Err(TexCompileError::UnsupportedUntilTectonicAdapter {
        entry_path: input.entry_path.clone(),
    })
}

#[must_use]
pub fn tex_pdf_artifact_path(entry_path: impl AsRef<Path>) -> PathBuf {
    entry_path.as_ref().with_extension("pdf")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{TexCompileError, TexCompileInput, compile_tex, tex_pdf_artifact_path};

    #[test]
    fn tex_pdf_artifact_path_replaces_extension() {
        assert_eq!(
            tex_pdf_artifact_path("resume.tex"),
            PathBuf::from("resume.pdf")
        );
        assert_eq!(
            tex_pdf_artifact_path("archive.resume.tex"),
            PathBuf::from("archive.resume.pdf")
        );
    }

    #[test]
    fn tex_compile_input_defaults_to_neighboring_pdf() {
        let input = TexCompileInput::new("docs/resume.tex");

        assert_eq!(input.entry_path(), PathBuf::from("docs/resume.tex"));
        assert_eq!(input.output_path(), PathBuf::from("docs/resume.pdf"));
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
    fn compile_tex_reports_adapter_gap() {
        let input = TexCompileInput::new("resume.tex");
        let error = compile_tex(&input).expect_err("compile should be deferred");

        assert_eq!(
            error,
            TexCompileError::UnsupportedUntilTectonicAdapter {
                entry_path: PathBuf::from("resume.tex")
            }
        );
        assert!(error.to_string().contains("Tectonic"));
    }
}
