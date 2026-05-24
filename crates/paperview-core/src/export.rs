use crate::{
    Document,
    parser::{Block, HeadingLevel, InlineSpan, ListItem, TableAlignment, TableCell},
};

use std::{error::Error, fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportFormat {
    Html,
    Pdf,
}

impl ExportFormat {
    #[must_use]
    pub fn extension(self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Pdf => "pdf",
        }
    }
}

impl FromStr for ExportFormat {
    type Err = ExportFormatParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "html" => Ok(Self::Html),
            "pdf" => Ok(Self::Pdf),
            _ => Err(ExportFormatParseError {
                format: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ExportFormatParseError {
    format: String,
}

impl fmt::Display for ExportFormatParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unsupported export format: {} (supported: html, pdf)",
            self.format
        )
    }
}

impl Error for ExportFormatParseError {}

#[derive(Debug, Eq, PartialEq)]
pub enum ExportError {
    PdfUnavailable,
}

impl fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PdfUnavailable => {
                write!(formatter, "PDF export is not available yet")
            }
        }
    }
}

impl Error for ExportError {}

#[derive(Debug, Eq, PartialEq)]
pub struct ExportArtifact {
    extension: &'static str,
    contents: Vec<u8>,
}

impl ExportArtifact {
    #[must_use]
    pub fn extension(&self) -> &'static str {
        self.extension
    }

    #[must_use]
    pub fn contents(&self) -> &[u8] {
        &self.contents
    }
}

pub fn export_document(
    document: &Document,
    format: ExportFormat,
) -> Result<ExportArtifact, ExportError> {
    match format {
        ExportFormat::Html => Ok(ExportArtifact {
            extension: format.extension(),
            contents: export_html(document).into_bytes(),
        }),
        ExportFormat::Pdf => Ok(ExportArtifact {
            extension: format.extension(),
            contents: export_pdf(document),
        }),
    }
}

#[must_use]
pub fn export_pdf(document: &Document) -> Vec<u8> {
    let lines = pdf_lines(document);
    write_pdf(&lines)
}

#[must_use]
pub fn export_html(document: &Document) -> String {
    let title = escape_text(document.title());
    let mut output = String::from("<!doctype html>\n<html>\n<head>\n");
    output.push_str("  <meta charset=\"utf-8\">\n");
    output.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    output.push_str(&format!("  <title>{title}</title>\n"));
    output.push_str("  <style>\n");
    output.push_str(CSS);
    output.push_str("  </style>\n</head>\n<body>\n<main class=\"paper\">\n");

    let heading_slugs = document
        .parsed()
        .toc()
        .into_iter()
        .map(|item| item.slug)
        .collect::<Vec<_>>();
    let mut heading_slugs = heading_slugs.iter();

    for block in &document.parsed().blocks {
        let heading_slug = matches!(block, Block::Heading { .. })
            .then(|| heading_slugs.next())
            .flatten()
            .map(String::as_str);
        render_block(block, heading_slug, &mut output);
    }

    output.push_str("</main>\n</body>\n</html>\n");
    output
}

fn render_block(block: &Block, heading_slug: Option<&str>, output: &mut String) {
    match block {
        Block::Heading { level, spans } => render_heading(*level, spans, heading_slug, output),
        Block::Paragraph(spans) => {
            output.push_str("<p>");
            render_inline_spans(spans, output);
            output.push_str("</p>\n");
        }
        Block::BlockQuote(spans) => {
            output.push_str("<blockquote class=\"callout\">");
            render_inline_spans(spans, output);
            output.push_str("</blockquote>\n");
        }
        Block::CodeBlock { language, code } => {
            render_code_block(language.as_deref(), code, "source-panel code-panel", output)
        }
        Block::Diagram { language, source } => {
            render_code_block(Some(language), source, "source-panel diagram-panel", output);
        }
        Block::Image { alt, url, title } => {
            output.push_str("<figure class=\"media-block\"><img src=\"");
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

fn render_heading(
    level: HeadingLevel,
    spans: &[InlineSpan],
    slug: Option<&str>,
    output: &mut String,
) {
    let depth = level.as_depth();
    output.push_str(&format!("<h{depth}"));
    if let Some(slug) = slug {
        output.push_str(" id=\"");
        output.push_str(&escape_attr(slug));
        output.push('"');
    }
    output.push('>');
    render_inline_spans(spans, output);
    output.push_str(&format!("</h{depth}>\n"));
}

fn render_code_block(language: Option<&str>, code: &str, block_class: &str, output: &mut String) {
    output.push_str("<pre class=\"");
    output.push_str(block_class);
    output.push_str("\"><code");
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
    output.push_str("<table class=\"data-table\">\n");
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
    let class = if items.iter().any(|item| item.checked.is_some()) {
        " class=\"task-list\""
    } else {
        ""
    };
    output.push_str(&format!("<{tag}{class}>\n"));
    for item in items {
        let item_class = if item.checked.is_some() {
            " class=\"task-item\""
        } else {
            ""
        };
        output.push_str(&format!("<li{item_class}>"));
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

#[derive(Debug, Clone, Eq, PartialEq)]
struct PdfLine {
    text: String,
    size: u16,
    indent: u16,
    gap_after: u16,
    outline_level: Option<u8>,
}

fn pdf_lines(document: &Document) -> Vec<PdfLine> {
    let mut lines = vec![
        pdf_line(document.title().to_owned(), 24)
            .with_indent(0)
            .with_gap_after(10),
    ];

    for block in &document.parsed().blocks {
        render_pdf_block(block, &mut lines);
    }

    lines
}

fn render_pdf_block(block: &Block, lines: &mut Vec<PdfLine>) {
    match block {
        Block::Heading { level, spans } => {
            lines.push(
                pdf_line(inline_plain_text(spans), pdf_heading_size(*level))
                    .with_gap_after(pdf_heading_gap(*level))
                    .with_outline(level.as_depth()),
            );
        }
        Block::Paragraph(spans) => push_wrapped_pdf_lines(&inline_plain_text(spans), 12, lines),
        Block::BlockQuote(spans) => {
            push_wrapped_pdf_lines_with_indent(
                &inline_plain_text(spans),
                12,
                18,
                PDF_BODY_WRAP_CHARS.saturating_sub(4),
                lines,
            );
        }
        Block::CodeBlock { language, code } => {
            if let Some(language) = language
                && !language.trim().is_empty()
            {
                lines.push(pdf_line(format!("code: {language}"), 10).with_gap_after(2));
            }
            push_preformatted_pdf_lines(code, lines);
        }
        Block::Diagram { language, source } => {
            lines.push(pdf_line(format!("diagram: {language}"), 10).with_gap_after(2));
            push_preformatted_pdf_lines(source, lines);
        }
        Block::Image { alt, url, title } => {
            let label = if alt.trim().is_empty() { "Image" } else { alt };
            push_wrapped_pdf_lines(&format!("[image] {label}: {url}"), 11, lines);
            if !title.trim().is_empty() {
                push_wrapped_pdf_lines(title, 10, lines);
            }
        }
        Block::Table { header, rows, .. } => {
            if !header.is_empty() {
                push_wrapped_pdf_lines(&pdf_table_row(header), 10, lines);
            }
            for row in rows {
                push_wrapped_pdf_lines(&pdf_table_row(row), 10, lines);
            }
        }
        Block::List { ordered, items } => {
            for (index, item) in items.iter().enumerate() {
                let marker = match item.checked {
                    Some(true) if *ordered => format!("{}. [x]", index + 1),
                    Some(false) if *ordered => format!("{}. [ ]", index + 1),
                    Some(true) => "- [x]".to_owned(),
                    Some(false) => "- [ ]".to_owned(),
                    None if *ordered => format!("{}.", index + 1),
                    None => "-".to_owned(),
                };
                push_wrapped_pdf_lines_with_indent(
                    &format!("{marker} {}", inline_plain_text(&item.content)),
                    12,
                    14,
                    PDF_BODY_WRAP_CHARS.saturating_sub(4),
                    lines,
                );
            }
        }
        Block::Math { display, source } => {
            let label = if *display {
                "display math"
            } else {
                "inline math"
            };
            lines.push(pdf_line(label.to_owned(), 10).with_gap_after(2));
            push_preformatted_pdf_lines(source, lines);
        }
        Block::Rule => lines.push(pdf_line("------------------------------".to_owned(), 10)),
    }

    if let Some(line) = lines.last_mut() {
        line.gap_after = line.gap_after.max(8);
    }
}

fn pdf_heading_size(level: HeadingLevel) -> u16 {
    match level {
        HeadingLevel::H1 => 20,
        HeadingLevel::H2 => 17,
        HeadingLevel::H3 => 15,
        HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => 13,
    }
}

fn pdf_heading_gap(level: HeadingLevel) -> u16 {
    match level {
        HeadingLevel::H1 => 10,
        HeadingLevel::H2 => 8,
        HeadingLevel::H3 => 7,
        HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => 6,
    }
}

const PDF_BODY_WRAP_CHARS: usize = 84;

impl PdfLine {
    fn with_indent(mut self, indent: u16) -> Self {
        self.indent = indent;
        self
    }

    fn with_gap_after(mut self, gap_after: u16) -> Self {
        self.gap_after = gap_after;
        self
    }

    fn with_outline(mut self, level: u8) -> Self {
        self.outline_level = Some(level);
        self
    }
}

fn pdf_line(text: String, size: u16) -> PdfLine {
    PdfLine {
        text,
        size,
        indent: 0,
        gap_after: 3,
        outline_level: None,
    }
}

fn inline_plain_text(spans: &[InlineSpan]) -> String {
    spans.iter().map(|span| span.text.as_str()).collect()
}

fn pdf_table_row(row: &[TableCell]) -> String {
    row.iter()
        .map(|cell| inline_plain_text(cell))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn push_wrapped_pdf_lines(text: &str, size: u16, lines: &mut Vec<PdfLine>) {
    push_wrapped_pdf_lines_with_indent(text, size, 0, PDF_BODY_WRAP_CHARS, lines);
}

fn push_wrapped_pdf_lines_with_indent(
    text: &str,
    size: u16,
    indent: u16,
    max_chars: usize,
    lines: &mut Vec<PdfLine>,
) {
    for wrapped in wrap_pdf_text(text, max_chars) {
        lines.push(pdf_line(wrapped, size).with_indent(indent));
    }
}

fn push_preformatted_pdf_lines(text: &str, lines: &mut Vec<PdfLine>) {
    for line in text.lines() {
        push_wrapped_pdf_lines_with_indent(
            line,
            10,
            14,
            PDF_BODY_WRAP_CHARS.saturating_sub(4),
            lines,
        );
    }
}

fn wrap_pdf_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let next_len =
            current.chars().count() + usize::from(!current.is_empty()) + word.chars().count();
        if next_len > max_chars && !current.is_empty() {
            lines.push(current);
            current = String::new();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }

    if current.is_empty() {
        lines.push(String::new());
    } else {
        lines.push(current);
    }

    lines
}

fn write_pdf(lines: &[PdfLine]) -> Vec<u8> {
    const PAGE_WIDTH: f32 = 612.0;
    const PAGE_HEIGHT: f32 = 792.0;
    const LEFT: f32 = 54.0;
    const TOP: f32 = 740.0;
    const BOTTOM: f32 = 54.0;

    let pages = paginate_pdf_lines(lines, TOP - BOTTOM);
    let page_count = pages.len().max(1);
    let mut objects = vec![
        String::new(),
        String::new(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
    ];
    let first_page_object = 4;
    let mut page_ids = Vec::new();
    let mut outlines = Vec::new();

    for (page_index, page_lines) in pages.iter().enumerate() {
        let page_id = first_page_object + (page_index * 2);
        let content_id = page_id + 1;
        page_ids.push(page_id);
        outlines.extend(pdf_page_outlines(page_lines, page_id, TOP));
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_WIDTH:.0} {PAGE_HEIGHT:.0}] /Resources << /Font << /F1 3 0 R >> >> /Contents {content_id} 0 R >>"
        ));
        let stream = pdf_page_stream(page_lines, LEFT, TOP);
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}endstream",
            stream.len(),
            stream
        ));
    }

    if page_ids.is_empty() {
        page_ids.push(first_page_object);
        let stream = pdf_page_stream(&[], LEFT, TOP);
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_WIDTH:.0} {PAGE_HEIGHT:.0}] /Resources << /Font << /F1 3 0 R >> >> /Contents 5 0 R >>"
        ));
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}endstream",
            stream.len(),
            stream
        ));
    }

    objects[1] = format!(
        "<< /Type /Pages /Kids [{}] /Count {page_count} >>",
        page_ids
            .iter()
            .map(|id| format!("{id} 0 R"))
            .collect::<Vec<_>>()
            .join(" ")
    );
    if outlines.is_empty() {
        objects[0] = "<< /Type /Catalog /Pages 2 0 R >>".to_owned();
    } else {
        let outline_root_id = objects.len() + 1;
        objects[0] = format!(
            "<< /Type /Catalog /Pages 2 0 R /Outlines {outline_root_id} 0 R /PageMode /UseOutlines >>"
        );
        append_pdf_outlines(&mut objects, outline_root_id, &outlines);
    }

    let mut pdf = Vec::from(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".as_slice());
    let mut offsets = vec![0usize];

    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, object).as_bytes());
    }

    let xref_offset = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );

    pdf
}

#[derive(Debug, Clone, PartialEq)]
struct PdfOutline {
    title: String,
    level: u8,
    page_id: usize,
    y: f32,
}

fn pdf_page_outlines(page_lines: &[PdfLine], page_id: usize, top: f32) -> Vec<PdfOutline> {
    let mut outlines = Vec::new();
    let mut y = top;

    for line in page_lines {
        if let Some(level) = line.outline_level {
            outlines.push(PdfOutline {
                title: line.text.clone(),
                level,
                page_id,
                y,
            });
        }
        y -= pdf_line_advance(line);
    }

    outlines
}

fn append_pdf_outlines(objects: &mut Vec<String>, outline_root_id: usize, outlines: &[PdfOutline]) {
    let first_item_id = outline_root_id + 1;
    let last_item_id = first_item_id + outlines.len() - 1;
    objects.push(format!(
        "<< /Type /Outlines /First {first_item_id} 0 R /Last {last_item_id} 0 R /Count {} >>",
        outlines.len()
    ));

    for (index, outline) in outlines.iter().enumerate() {
        let object_id = first_item_id + index;
        let previous = (index > 0).then(|| format!("/Prev {} 0 R ", object_id - 1));
        let next = (index + 1 < outlines.len()).then(|| format!("/Next {} 0 R ", object_id + 1));
        let title = format!(
            "{}{}",
            "  ".repeat(usize::from(outline.level.saturating_sub(1))),
            outline.title
        );

        objects.push(format!(
            "<< /Title ({}) /Parent {outline_root_id} 0 R {}{} /Dest [{} 0 R /XYZ 54.0 {:.1} null] >>",
            escape_pdf_text(&title),
            previous.unwrap_or_default(),
            next.unwrap_or_default(),
            outline.page_id,
            outline.y
        ));
    }
}

fn paginate_pdf_lines(lines: &[PdfLine], available_height: f32) -> Vec<&[PdfLine]> {
    if lines.is_empty() {
        return Vec::new();
    }

    let mut pages = Vec::new();
    let mut start = 0;
    let mut used = 0.0;

    for (index, line) in lines.iter().enumerate() {
        let advance = pdf_line_advance(line);
        if index > start && used + advance > available_height {
            pages.push(&lines[start..index]);
            start = index;
            used = 0.0;
        }
        used += advance;
    }

    pages.push(&lines[start..]);
    pages
}

fn pdf_line_advance(line: &PdfLine) -> f32 {
    (f32::from(line.size) * 1.25) + f32::from(line.gap_after)
}

fn pdf_page_stream(lines: &[PdfLine], left: f32, top: f32) -> String {
    let mut stream = String::new();
    let mut y = top;

    for line in lines {
        let x = left + f32::from(line.indent);
        stream.push_str(&format!(
            "BT /F1 {} Tf 1 0 0 1 {x:.1} {y:.1} Tm ({}) Tj ET\n",
            line.size,
            escape_pdf_text(&line.text)
        ));
        y -= pdf_line_advance(line);
    }

    stream
}

fn escape_pdf_text(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            '(' => "\\(".to_owned(),
            ')' => "\\)".to_owned(),
            '\\' => "\\\\".to_owned(),
            character if character.is_ascii() && !character.is_control() => character.to_string(),
            _ => "?".to_owned(),
        })
        .collect()
}

const CSS: &str = r#"    :root {
      color-scheme: light;
      --shell-bg: #111318;
      --paper-bg: #fdf8ef;
      --reader-text: #1f2328;
      --muted-text: #525862;
      --accent: #58a6ff;
      --border: #d0d7de;
      --panel-bg: #f6f8fa;
      --math-accent: #d29922;
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background: var(--shell-bg);
      color: var(--reader-text);
    }
    body {
      margin: 0;
      background: var(--shell-bg);
    }
    .paper {
      box-sizing: border-box;
      max-width: 900px;
      min-height: 100vh;
      margin: 0 auto;
      padding: 56px;
      background: var(--paper-bg);
      box-shadow: 0 0 0 1px rgba(208, 215, 222, 0.35), 0 24px 80px rgba(0, 0, 0, 0.32);
    }
    h1, h2, h3, h4, h5, h6 {
      line-height: 1.2;
      margin: 1.25em 0 0.55em;
      letter-spacing: 0;
    }
    h1 {
      margin-top: 0;
      padding-bottom: 0.35em;
      border-bottom: 1px solid var(--border);
    }
    p, blockquote, li {
      line-height: 1.65;
    }
    a {
      color: #0969da;
      text-decoration-thickness: 0.08em;
      text-underline-offset: 0.18em;
    }
    .callout {
      margin: 1em 0;
      padding: 0.65em 1em;
      border-left: 4px solid var(--accent);
      color: var(--muted-text);
      background: var(--panel-bg);
    }
    .source-panel {
      overflow: auto;
      padding: 1em;
      border: 1px solid var(--border);
      border-radius: 6px;
      background: var(--panel-bg);
    }
    .diagram-panel {
      border-left: 4px solid var(--accent);
    }
    code {
      font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
      font-size: 0.92em;
    }
    :not(pre) > code {
      padding: 0.1em 0.3em;
      border-radius: 4px;
      background: var(--panel-bg);
    }
    .data-table {
      width: 100%;
      border-collapse: collapse;
      margin: 1em 0;
    }
    th, td {
      padding: 0.55em 0.7em;
      border: 1px solid var(--border);
      text-align: left;
      vertical-align: top;
    }
    th {
      background: var(--panel-bg);
    }
    .align-center {
      text-align: center;
    }
    .align-right {
      text-align: right;
    }
    .media-block {
      margin: 1.25em 0;
    }
    img {
      max-width: 100%;
      border-radius: 6px;
    }
    figcaption {
      margin-top: 0.4em;
      color: var(--muted-text);
      font-size: 0.9em;
    }
    .task-list {
      list-style: none;
      padding-left: 0;
    }
    .task-item input {
      margin-right: 0.45em;
      accent-color: var(--accent);
    }
    .math {
      border-left: 4px solid var(--math-accent);
    }
    @media (max-width: 720px) {
      .paper {
        padding: 28px;
      }
    }
"#;

#[cfg(test)]
mod tests {
    use crate::{
        Document,
        export::{ExportFormat, export_document, export_html, export_pdf},
    };

    #[test]
    fn exports_basic_html_document() {
        let document = Document::from_source(
            "# PaperView\n\nA **native** [reader](docs/index.md).\n\n- [x] Done",
        );
        let html = export_html(&document);

        assert!(html.contains("<title>PaperView</title>"));
        assert!(html.contains("<main class=\"paper\">"));
        assert!(html.contains("<h1 id=\"paperview\">PaperView</h1>"));
        assert!(html.contains("<strong>native</strong>"));
        assert!(html.contains("<a href=\"docs/index.md\">reader</a>"));
        assert!(html.contains("<ul class=\"task-list\">"));
        assert!(html.contains("<li class=\"task-item\">"));
        assert!(html.contains("<input type=\"checkbox\" disabled checked> Done"));
    }

    #[test]
    fn exports_paperview_html_theme_styles() {
        let document = Document::from_source(
            "# PaperView\n\n> Note\n\n```mermaid\ngraph TD\n  A-->B\n```\n\n$$ E = mc^2 $$",
        );
        let html = export_html(&document);

        assert!(html.contains("--shell-bg: #111318;"));
        assert!(html.contains("--paper-bg: #fdf8ef;"));
        assert!(html.contains(".source-panel"));
        assert!(html.contains("<blockquote class=\"callout\">"));
        assert!(html.contains(
            "<pre class=\"source-panel diagram-panel\"><code class=\"language-mermaid\">"
        ));
        assert!(html.contains("<pre class=\"math display\"><code>"));
    }

    #[test]
    fn escapes_html_text_and_attributes() {
        let document = Document::from_source("# PaperView & Friends\n\n[link](docs/\"quote\".md)");
        let html = export_html(&document);

        assert!(html.contains("<h1 id=\"paperview-friends\">PaperView &amp; Friends</h1>"));
        assert!(html.contains("href=\"docs/&quot;quote&quot;.md\""));
    }

    #[test]
    fn exports_duplicate_heading_anchors() {
        let document = Document::from_source("# Intro\n\n## Details\n\n### Details");
        let html = export_html(&document);

        assert!(html.contains("<h1 id=\"intro\">Intro</h1>"));
        assert!(html.contains("<h2 id=\"details\">Details</h2>"));
        assert!(html.contains("<h3 id=\"details-2\">Details</h3>"));
    }

    #[test]
    fn parses_export_formats() {
        assert_eq!("html".parse::<ExportFormat>(), Ok(ExportFormat::Html));
        assert_eq!("PDF".parse::<ExportFormat>(), Ok(ExportFormat::Pdf));
        assert!("docx".parse::<ExportFormat>().is_err());
    }

    #[test]
    fn exports_html_artifact() {
        let document = Document::from_source("# PaperView");
        let artifact = export_document(&document, ExportFormat::Html).expect("html export");

        assert_eq!(artifact.extension(), "html");
        assert!(artifact.contents().starts_with(b"<!doctype html>"));
    }

    #[test]
    fn exports_pdf_artifact() {
        let document = Document::from_source("# PaperView");
        let artifact = export_document(&document, ExportFormat::Pdf).expect("pdf export");

        assert_eq!(artifact.extension(), "pdf");
        assert!(artifact.contents().starts_with(b"%PDF-1.4"));
        assert!(artifact.contents().ends_with(b"%%EOF\n"));
    }

    #[test]
    fn exports_pdf_text_content() {
        let document =
            Document::from_source("# PaperView\n\n- [x] Done\n\n```rust\nfn main() {}\n```");
        let pdf = export_pdf(&document);
        let pdf_text = String::from_utf8_lossy(&pdf);

        assert!(pdf_text.contains("BT /F1 24 Tf"));
        assert!(pdf_text.contains("(PaperView) Tj"));
        assert!(pdf_text.contains("1 0 0 1 68.0"));
        assert!(pdf_text.contains("(- [x] Done) Tj"));
        assert!(pdf_text.contains("(code: rust) Tj"));
        assert!(pdf_text.contains("(fn main\\(\\) {}) Tj"));
    }

    #[test]
    fn exports_pdf_heading_outlines() {
        let document = Document::from_source("# Intro\n\nBody.\n\n## Details\n\nMore.");
        let pdf = export_pdf(&document);
        let pdf_text = String::from_utf8_lossy(&pdf);

        assert!(pdf_text.contains("/Outlines"));
        assert!(pdf_text.contains("/PageMode /UseOutlines"));
        assert!(pdf_text.contains("/Title (Intro)"));
        assert!(pdf_text.contains("/Title (  Details)"));
        assert!(pdf_text.contains("/Dest [4 0 R /XYZ 54.0"));
    }

    #[test]
    fn exports_pdf_with_multiple_pages_for_long_documents() {
        let source = format!(
            "# Long\n\n{}",
            (0..90)
                .map(|index| format!("Paragraph {index} with enough text to render."))
                .collect::<Vec<_>>()
                .join("\n\n")
        );
        let pdf = export_pdf(&Document::from_source(source));
        let pdf_text = String::from_utf8_lossy(&pdf);

        assert!(pdf_text.contains("/Count 3") || pdf_text.contains("/Count 4"));
    }
}
