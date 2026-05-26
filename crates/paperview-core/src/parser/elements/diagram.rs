pub const MERMAID_LANGUAGE: &str = "mermaid";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowchartPreview {
    pub direction: FlowchartDirection,
    pub edges: Vec<FlowchartEdge>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowchartDirection {
    TopDown,
    BottomTop,
    LeftRight,
    RightLeft,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowchartEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

#[must_use]
pub fn is_mermaid(language: Option<&str>) -> bool {
    language
        .and_then(|language| language.split_whitespace().next())
        .is_some_and(|language| language.eq_ignore_ascii_case(MERMAID_LANGUAGE))
}

#[must_use]
pub fn source(source: &str) -> String {
    source.trim().to_owned()
}

#[must_use]
pub fn flowchart_preview(source: &str) -> Option<FlowchartPreview> {
    let mut lines = source
        .lines()
        .filter_map(clean_line)
        .filter(|line| !line.is_empty() && !line.starts_with("%%"));

    let header = lines.next()?;
    let direction = parse_flowchart_header(header)?;
    let edges = lines.filter_map(parse_edge).collect::<Vec<_>>();

    (!edges.is_empty()).then_some(FlowchartPreview { direction, edges })
}

fn parse_flowchart_header(line: &str) -> Option<FlowchartDirection> {
    let line = line.trim_end_matches(';');
    let mut parts = line.split_whitespace();
    let keyword = parts.next()?;

    if !keyword.eq_ignore_ascii_case("graph") && !keyword.eq_ignore_ascii_case("flowchart") {
        return None;
    }

    parts.next().map_or(
        Some(FlowchartDirection::TopDown),
        |direction| match direction.to_ascii_uppercase().as_str() {
            "TD" | "TB" => Some(FlowchartDirection::TopDown),
            "BT" => Some(FlowchartDirection::BottomTop),
            "LR" => Some(FlowchartDirection::LeftRight),
            "RL" => Some(FlowchartDirection::RightLeft),
            _ => None,
        },
    )
}

fn parse_edge(line: &str) -> Option<FlowchartEdge> {
    let line = line.trim_end_matches(';').trim();
    if let Some(edge) = parse_labeled_edge(line) {
        return Some(edge);
    }

    let (arrow, left, right) = split_edge(line)?;
    let (label, to) = parse_edge_label(arrow, right);

    Some(FlowchartEdge {
        from: parse_node(left),
        to: parse_node(to),
        label,
    })
}

fn clean_line(line: &str) -> Option<&str> {
    let line = line.trim();
    let line = line
        .split_once("%%")
        .map_or(line, |(before_comment, _)| before_comment.trim());

    (!line.is_empty()).then_some(line)
}

fn parse_labeled_edge(line: &str) -> Option<FlowchartEdge> {
    for (start, end) in [("--", "-->"), ("-.", ".->"), ("==", "==>")] {
        let Some((left, rest)) = line.split_once(start) else {
            continue;
        };
        let Some((label, right)) = rest.split_once(end) else {
            continue;
        };
        if label.trim().is_empty() {
            continue;
        }

        return Some(FlowchartEdge {
            from: parse_node(left),
            to: parse_node(right),
            label: clean_label(label),
        });
    }

    None
}

fn split_edge(line: &str) -> Option<(&str, &str, &str)> {
    const ARROWS: [&str; 8] = ["-->", "---", "-.->", "-.-", "==>", "===", "~~~", "--"];

    ARROWS.iter().find_map(|arrow| {
        let (left, right) = line.split_once(arrow)?;
        Some((*arrow, left.trim(), right.trim()))
    })
}

fn parse_edge_label<'a>(arrow: &str, right: &'a str) -> (Option<String>, &'a str) {
    if let Some(rest) = right.strip_prefix('|')
        && let Some((label, to)) = rest.split_once('|')
    {
        return (clean_label(label), to.trim());
    }

    if let Some((label, to)) = right.split_once("-->") {
        return (clean_label(label), to.trim());
    }
    if let Some((label, to)) = right.split_once("-.->") {
        return (clean_label(label), to.trim());
    }
    if let Some((label, to)) = right.split_once("==>") {
        return (clean_label(label), to.trim());
    }

    if arrow == "---" || arrow == "--" {
        if let Some((label, to)) = right.split_once("---") {
            return (clean_label(label), to.trim());
        }
        if let Some((label, to)) = right.split_once("--") {
            return (clean_label(label), to.trim());
        }
    }

    (None, right)
}

fn clean_label(raw: &str) -> Option<String> {
    let label = raw.trim().trim_matches('|').trim();
    (!label.is_empty()).then(|| label.to_owned())
}

fn parse_node(raw: &str) -> String {
    let raw = raw
        .trim()
        .split_once(":::")
        .map_or(raw.trim(), |(node, _)| node.trim());
    let label_start = raw.find(['[', '(', '{']).unwrap_or(raw.len());
    let id = raw[..label_start].trim();
    let label = raw[label_start..].trim().trim_matches(|character: char| {
        matches!(
            character,
            '[' | ']' | '(' | ')' | '{' | '}' | '/' | '\\' | '"' | '\'' | ' '
        )
    });

    let text = if label.is_empty() { id } else { label };

    text.to_owned()
}

#[cfg(test)]
mod tests {
    use super::{FlowchartDirection, FlowchartEdge, FlowchartPreview, flowchart_preview};

    #[test]
    fn parses_simple_flowchart_edges() {
        let preview = flowchart_preview("graph TD\n  A[Start] --> B{Decision}\n  B --> C[Done]")
            .expect("flowchart preview");

        assert_eq!(
            preview,
            FlowchartPreview {
                direction: FlowchartDirection::TopDown,
                edges: vec![
                    FlowchartEdge {
                        from: "Start".to_owned(),
                        to: "Decision".to_owned(),
                        label: None
                    },
                    FlowchartEdge {
                        from: "B".to_owned(),
                        to: "Done".to_owned(),
                        label: None
                    }
                ]
            }
        );
    }

    #[test]
    fn parses_labeled_flowchart_edges() {
        let preview = flowchart_preview(
            "flowchart LR\n  A -- yes --> B\n  B -. maybe .-> C\n  C ==>|fast| D[Done]",
        )
        .expect("flowchart preview");

        assert_eq!(
            preview,
            FlowchartPreview {
                direction: FlowchartDirection::LeftRight,
                edges: vec![
                    FlowchartEdge {
                        from: "A".to_owned(),
                        to: "B".to_owned(),
                        label: Some("yes".to_owned())
                    },
                    FlowchartEdge {
                        from: "B".to_owned(),
                        to: "C".to_owned(),
                        label: Some("maybe".to_owned())
                    },
                    FlowchartEdge {
                        from: "C".to_owned(),
                        to: "Done".to_owned(),
                        label: Some("fast".to_owned())
                    }
                ]
            }
        );
    }

    #[test]
    fn parses_common_flowchart_node_shapes_and_comments() {
        let preview = flowchart_preview(
            "flowchart TD;\n  %% startup path\n  A((Start)):::entry --> B[/Input/] %% inline note\n  B --> C[(Store)]\n  C --> D{{Done}}",
        )
        .expect("flowchart preview");

        assert_eq!(
            preview,
            FlowchartPreview {
                direction: FlowchartDirection::TopDown,
                edges: vec![
                    FlowchartEdge {
                        from: "Start".to_owned(),
                        to: "Input".to_owned(),
                        label: None
                    },
                    FlowchartEdge {
                        from: "B".to_owned(),
                        to: "Store".to_owned(),
                        label: None
                    },
                    FlowchartEdge {
                        from: "C".to_owned(),
                        to: "Done".to_owned(),
                        label: None
                    }
                ]
            }
        );
    }

    #[test]
    fn ignores_unsupported_mermaid_diagrams() {
        assert_eq!(flowchart_preview("sequenceDiagram\nA->>B: Hello"), None);
        assert_eq!(flowchart_preview("graph TD\n  A"), None);
    }
}
