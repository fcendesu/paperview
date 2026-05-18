use crate::{
    Document,
    parser::{Block, HeadingLevel, InlineSpan, ListItem, TableAlignment, TableCell},
};

#[must_use]
pub fn export_html(document: &Document) -> String {
    let title = escape_text(document.title());
    let mut output = String::from("<!doctype html>\n<html>\n<head>\n");
    output.push_str("  <meta charset=\"utf-8\">\n");
    output.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    output.push_str(&format!("  <title>{title}</title>\n"));
    output.push_str("  <style>\n");
    output.push_str(CSS);
    output.push_str("  </style>\n</head>\n<body>\n<main>\n");

    for block in &document.parsed().blocks {
        render_block(block, &mut output);
    }

    output.push_str("</main>\n</body>\n</html>\n");
    output
}

fn render_block(block: &Block, output: &mut String) {
    match block {
        Block::Heading { level, spans } => render_heading(*level, spans, output),
        Block::Paragraph(spans) => {
            output.push_str("<p>");
            render_inline_spans(spans, output);
            output.push_str("</p>\n");
        }
        Block::BlockQuote(spans) => {
            output.push_str("<blockquote>");
            render_inline_spans(spans, output);
            output.push_str("</blockquote>\n");
        }
        Block::CodeBlock { language, code } => render_code_block(language.as_deref(), code, output),
        Block::Diagram { language, source } => render_code_block(Some(language), source, output),
        Block::Image { alt, url, title } => {
            output.push_str("<figure><img src=\"");
            output.push_str(&escape_attr(url));
            output.push_str("\" alt=\"");
            output.push_str(&escape_attr(alt));
            if !title.is_empty() {
                output.push_str("\" title=\"");
                output.push_str(&escape_attr(title));
            }
            output.push_str("\">");
            if !alt.is_empty() {
                output.push_str("<figcaption>");
                output.push_str(&escape_text(alt));
                output.push_str("</figcaption>");
            }
            output.push_str("</figure>\n");
        }
        Block::Table {
            alignments,
            header,
            rows,
        } => render_table(alignments, header, rows, output),
        Block::List { ordered, items } => render_list(*ordered, items, output),
        Block::Math { display, source } => {
            let class = if *display {
                "math display"
            } else {
                "math inline"
            };
            output.push_str(&format!("<pre class=\"{class}\"><code>"));
            output.push_str(&escape_text(source));
            output.push_str("</code></pre>\n");
        }
        Block::Rule => output.push_str("<hr>\n"),
    }
}

fn render_heading(level: HeadingLevel, spans: &[InlineSpan], output: &mut String) {
    let depth = level.as_depth();
    output.push_str(&format!("<h{depth}>"));
    render_inline_spans(spans, output);
    output.push_str(&format!("</h{depth}>\n"));
}

fn render_code_block(language: Option<&str>, code: &str, output: &mut String) {
    output.push_str("<pre><code");
    if let Some(language) = language
        && !language.trim().is_empty()
    {
        output.push_str(" class=\"language-");
        output.push_str(&escape_attr(language));
        output.push('"');
    }
    output.push('>');
    output.push_str(&escape_text(code));
    output.push_str("</code></pre>\n");
}

fn render_table(
    alignments: &[TableAlignment],
    header: &[TableCell],
    rows: &[Vec<TableCell>],
    output: &mut String,
) {
    output.push_str("<table>\n");
    if !header.is_empty() {
        output.push_str("<thead><tr>");
        for (index, cell) in header.iter().enumerate() {
            render_table_cell("th", cell, alignments.get(index), output);
        }
        output.push_str("</tr></thead>\n");
    }
    if !rows.is_empty() {
        output.push_str("<tbody>\n");
        for row in rows {
            output.push_str("<tr>");
            for (index, cell) in row.iter().enumerate() {
                render_table_cell("td", cell, alignments.get(index), output);
            }
            output.push_str("</tr>\n");
        }
        output.push_str("</tbody>\n");
    }
    output.push_str("</table>\n");
}

fn render_table_cell(
    tag: &str,
    cell: &[InlineSpan],
    alignment: Option<&TableAlignment>,
    output: &mut String,
) {
    output.push_str(&format!("<{tag}"));
    if let Some(alignment) = alignment_class(alignment.copied().unwrap_or(TableAlignment::None)) {
        output.push_str(" class=\"");
        output.push_str(alignment);
        output.push('"');
    }
    output.push('>');
    render_inline_spans(cell, output);
    output.push_str(&format!("</{tag}>"));
}

fn alignment_class(alignment: TableAlignment) -> Option<&'static str> {
    match alignment {
        TableAlignment::None => None,
        TableAlignment::Left => Some("align-left"),
        TableAlignment::Center => Some("align-center"),
        TableAlignment::Right => Some("align-right"),
    }
}

fn render_list(ordered: bool, items: &[ListItem], output: &mut String) {
    let tag = if ordered { "ol" } else { "ul" };
    output.push_str(&format!("<{tag}>\n"));
    for item in items {
        output.push_str("<li>");
        if let Some(checked) = item.checked {
            output.push_str("<input type=\"checkbox\" disabled");
            if checked {
                output.push_str(" checked");
            }
            output.push_str("> ");
        }
        render_inline_spans(&item.content, output);
        output.push_str("</li>\n");
    }
    output.push_str(&format!("</{tag}>\n"));
}

fn render_inline_spans(spans: &[InlineSpan], output: &mut String) {
    for span in spans {
        render_inline_span(span, output);
    }
}

fn render_inline_span(span: &InlineSpan, output: &mut String) {
    if let Some(link) = &span.link {
        output.push_str("<a href=\"");
        output.push_str(&escape_attr(link));
        output.push_str("\">");
    }
    if span.strong {
        output.push_str("<strong>");
    }
    if span.emphasis {
        output.push_str("<em>");
    }
    if span.code {
        output.push_str("<code>");
    }

    output.push_str(&escape_text(&span.text));

    if span.code {
        output.push_str("</code>");
    }
    if span.emphasis {
        output.push_str("</em>");
    }
    if span.strong {
        output.push_str("</strong>");
    }
    if span.link.is_some() {
        output.push_str("</a>");
    }
}

fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(value: &str) -> String {
    escape_text(value).replace('"', "&quot;")
}

const CSS: &str = r#"    :root {
      color-scheme: light;
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background: #111318;
      color: #1f2328;
    }
    body {
      margin: 0;
      background: #111318;
    }
    main {
      box-sizing: border-box;
      max-width: 860px;
      min-height: 100vh;
      margin: 0 auto;
      padding: 56px;
      background: #fdf8ef;
    }
    h1, h2, h3, h4, h5, h6 {
      line-height: 1.2;
      margin: 1.2em 0 0.55em;
    }
    p, blockquote, li {
      line-height: 1.6;
    }
    blockquote {
      margin: 1em 0;
      padding: 0.5em 1em;
      border-left: 4px solid #58a6ff;
      color: #525862;
      background: #f6f8fa;
    }
    pre {
      overflow: auto;
      padding: 1em;
      border: 1px solid #d0d7de;
      border-radius: 6px;
      background: #f6f8fa;
    }
    code {
      font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    }
    :not(pre) > code {
      padding: 0.1em 0.3em;
      border-radius: 4px;
      background: #f6f8fa;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      margin: 1em 0;
    }
    th, td {
      padding: 0.55em 0.7em;
      border: 1px solid #d0d7de;
      text-align: left;
    }
    th {
      background: #f6f8fa;
    }
    .align-center {
      text-align: center;
    }
    .align-right {
      text-align: right;
    }
    figure {
      margin: 1em 0;
    }
    img {
      max-width: 100%;
    }
    figcaption {
      margin-top: 0.4em;
      color: #525862;
      font-size: 0.9em;
    }
    .math {
      border-left: 4px solid #d29922;
    }
"#;

#[cfg(test)]
mod tests {
    use crate::{Document, export::export_html};

    #[test]
    fn exports_basic_html_document() {
        let document = Document::from_source(
            "# PaperView\n\nA **native** [reader](docs/index.md).\n\n- [x] Done",
        );
        let html = export_html(&document);

        assert!(html.contains("<title>PaperView</title>"));
        assert!(html.contains("<h1>PaperView</h1>"));
        assert!(html.contains("<strong>native</strong>"));
        assert!(html.contains("<a href=\"docs/index.md\">reader</a>"));
        assert!(html.contains("<input type=\"checkbox\" disabled checked> Done"));
    }

    #[test]
    fn escapes_html_text_and_attributes() {
        let document = Document::from_source("# PaperView & Friends\n\n[link](docs/\"quote\".md)");
        let html = export_html(&document);

        assert!(html.contains("<h1>PaperView &amp; Friends</h1>"));
        assert!(html.contains("href=\"docs/&quot;quote&quot;.md\""));
    }
}
