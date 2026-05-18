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
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("%%"));

    let header = lines.next()?;
    let direction = parse_flowchart_header(header)?;
    let edges = lines.filter_map(parse_edge).collect::<Vec<_>>();

    (!edges.is_empty()).then_some(FlowchartPreview { direction, edges })
}

fn parse_flowchart_header(line: &str) -> Option<FlowchartDirection> {
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
    let (_arrow, left, right) = split_edge(line)?;

    Some(FlowchartEdge {
        from: parse_node(left),
        to: parse_node(right),
        label: None,
    })
}

fn split_edge(line: &str) -> Option<(&str, &str, &str)> {
    const ARROWS: [&str; 5] = ["-->", "---", "-.->", "==>", "~~~"];

    ARROWS.iter().find_map(|arrow| {
        let (left, right) = line.split_once(arrow)?;
        Some((*arrow, left.trim(), right.trim()))
    })
}

fn parse_node(raw: &str) -> String {
    let raw = raw.trim();
    let label_start = raw.find(['[', '(', '{']).unwrap_or(raw.len());
    let id = raw[..label_start].trim();
    let label = raw[label_start..].trim().trim_matches(|character: char| {
        matches!(
            character,
            '[' | ']' | '(' | ')' | '{' | '}' | '"' | '\'' | ' '
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
    fn ignores_unsupported_mermaid_diagrams() {
        assert_eq!(flowchart_preview("sequenceDiagram\nA->>B: Hello"), None);
        assert_eq!(flowchart_preview("graph TD\n  A"), None);
    }
}
