use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    path: Option<PathBuf>,
    title: String,
    source: String,
}

impl Document {
    #[must_use]
    pub fn from_source(source: impl Into<String>) -> Self {
        let source = source.into();
        let title = parser_title(&source).unwrap_or_else(|| "Untitled".to_owned());

        Self {
            path: None,
            title,
            source,
        }
    }

    #[must_use]
    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    #[must_use]
    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }
}

fn parser_title(source: &str) -> Option<String> {
    source
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::Document;

    #[test]
    fn uses_first_h1_as_title() {
        let document = Document::from_source("# PaperView\n\nNative Markdown viewer.");

        assert_eq!(document.title(), "PaperView");
    }

    #[test]
    fn falls_back_to_untitled_without_h1() {
        let document = Document::from_source("Native Markdown viewer.");

        assert_eq!(document.title(), "Untitled");
    }
}
