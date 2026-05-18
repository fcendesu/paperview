use std::{
    fmt, io,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub line_index: usize,
    pub column: usize,
    pub line: String,
}

#[must_use]
pub fn search_lines(source: &str, query: &str) -> Vec<SearchMatch> {
    let needle = query.trim();
    if needle.is_empty() {
        return Vec::new();
    }

    let needle = needle.to_lowercase();

    source
        .lines()
        .enumerate()
        .filter_map(|(line_index, line)| {
            let lower_line = line.to_lowercase();
            lower_line.find(&needle).map(|byte_index| SearchMatch {
                line_index,
                column: line[..byte_index].chars().count(),
                line: line.to_owned(),
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSearchMatch {
    pub path: PathBuf,
    pub line_number: usize,
    pub column: usize,
    pub line: String,
}

pub fn search_workspace(
    query: &str,
    root: impl AsRef<Path>,
) -> Result<Vec<WorkspaceSearchMatch>, WorkspaceSearchError> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let output = Command::new("rg")
        .args(["--vimgrep", "--line-number", "--column", "--smart-case"])
        .arg(query)
        .arg(root.as_ref())
        .output()
        .map_err(|source| WorkspaceSearchError::RipgrepFailed { source })?;

    if output.status.success() || output.status.code() == Some(1) {
        parse_ripgrep_vimgrep(&String::from_utf8_lossy(&output.stdout))
    } else {
        Err(WorkspaceSearchError::RipgrepExited {
            status: output.status.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        })
    }
}

#[derive(Debug)]
pub enum WorkspaceSearchError {
    RipgrepFailed { source: io::Error },
    RipgrepExited { status: String, stderr: String },
    InvalidOutput { line: String },
}

impl fmt::Display for WorkspaceSearchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RipgrepFailed { source } => write!(formatter, "failed to run rg: {source}"),
            Self::RipgrepExited { status, stderr } if stderr.is_empty() => {
                write!(formatter, "rg exited with {status}")
            }
            Self::RipgrepExited { status, stderr } => {
                write!(formatter, "rg exited with {status}: {stderr}")
            }
            Self::InvalidOutput { line } => write!(formatter, "invalid rg output line: {line}"),
        }
    }
}

impl std::error::Error for WorkspaceSearchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RipgrepFailed { source } => Some(source),
            Self::RipgrepExited { .. } | Self::InvalidOutput { .. } => None,
        }
    }
}

fn parse_ripgrep_vimgrep(raw: &str) -> Result<Vec<WorkspaceSearchMatch>, WorkspaceSearchError> {
    raw.lines().map(parse_ripgrep_line).collect()
}

fn parse_ripgrep_line(line: &str) -> Result<WorkspaceSearchMatch, WorkspaceSearchError> {
    let mut parts = line.splitn(4, ':');
    let path = parts
        .next()
        .ok_or_else(|| WorkspaceSearchError::InvalidOutput {
            line: line.to_owned(),
        })?;
    let line_number = parts
        .next()
        .and_then(|part| part.parse().ok())
        .ok_or_else(|| WorkspaceSearchError::InvalidOutput {
            line: line.to_owned(),
        })?;
    let column = parts
        .next()
        .and_then(|part| part.parse().ok())
        .ok_or_else(|| WorkspaceSearchError::InvalidOutput {
            line: line.to_owned(),
        })?;
    let text = parts
        .next()
        .ok_or_else(|| WorkspaceSearchError::InvalidOutput {
            line: line.to_owned(),
        })?;

    Ok(WorkspaceSearchMatch {
        path: PathBuf::from(path),
        line_number,
        column,
        line: text.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::{WorkspaceSearchMatch, parse_ripgrep_vimgrep, search_lines};

    #[test]
    fn finds_case_insensitive_line_matches() {
        let matches = search_lines("# PaperView\n\nNative paper reader.", "paper");

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line_index, 0);
        assert_eq!(matches[0].column, 2);
        assert_eq!(matches[1].line_index, 2);
        assert_eq!(matches[1].column, 7);
    }

    #[test]
    fn ignores_empty_queries() {
        assert!(search_lines("PaperView", " ").is_empty());
    }

    #[test]
    fn parses_ripgrep_vimgrep_output() {
        let matches = parse_ripgrep_vimgrep("docs/PRD.md:1:3:PaperView\nREADME.md:9:1:PaperView")
            .expect("parse rg output");

        assert_eq!(
            matches,
            vec![
                WorkspaceSearchMatch {
                    path: "docs/PRD.md".into(),
                    line_number: 1,
                    column: 3,
                    line: "PaperView".to_owned()
                },
                WorkspaceSearchMatch {
                    path: "README.md".into(),
                    line_number: 9,
                    column: 1,
                    line: "PaperView".to_owned()
                }
            ]
        );
    }

    #[test]
    fn rejects_invalid_ripgrep_output() {
        assert!(parse_ripgrep_vimgrep("not enough parts").is_err());
    }
}
