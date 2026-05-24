use crate::parser::InlineSpan;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InlineState {
    pub strong_depth: usize,
    pub emphasis_depth: usize,
    pub links: Vec<String>,
}

impl InlineState {
    #[must_use]
    pub fn strong(&self) -> bool {
        self.strong_depth > 0
    }

    #[must_use]
    pub fn emphasis(&self) -> bool {
        self.emphasis_depth > 0
    }

    #[must_use]
    pub fn link(&self) -> Option<String> {
        self.links.last().cloned()
    }
}

#[must_use]
pub fn span(text: &str, state: &InlineState) -> InlineSpan {
    InlineSpan {
        text: text.to_owned(),
        strong: state.strong(),
        emphasis: state.emphasis(),
        code: false,
        math: false,
        link: state.link(),
    }
}

#[must_use]
pub fn code_span(text: &str, state: &InlineState) -> InlineSpan {
    InlineSpan {
        text: text.to_owned(),
        strong: state.strong(),
        emphasis: state.emphasis(),
        code: true,
        math: false,
        link: state.link(),
    }
}

#[must_use]
pub fn math_span(text: &str, state: &InlineState) -> InlineSpan {
    InlineSpan {
        text: text.to_owned(),
        strong: state.strong(),
        emphasis: state.emphasis(),
        code: false,
        math: true,
        link: state.link(),
    }
}

pub fn push_span(spans: &mut Vec<InlineSpan>, next: InlineSpan) {
    if let Some(previous) = spans.last_mut()
        && previous.strong == next.strong
        && previous.emphasis == next.emphasis
        && previous.code == next.code
        && previous.math == next.math
        && previous.link == next.link
    {
        previous.text.push_str(&next.text);
        return;
    }

    spans.push(next);
}

#[must_use]
pub fn plain_text(spans: &[InlineSpan]) -> String {
    spans.iter().map(|span| span.text.as_str()).collect()
}

#[must_use]
pub fn markdown_text(spans: &[InlineSpan]) -> String {
    spans.iter().map(markdown_span).collect()
}

fn markdown_span(span: &InlineSpan) -> String {
    let mut text = span.text.clone();

    if span.code {
        text = format!("`{text}`");
    }
    if span.math && !text.starts_with('$') {
        text = format!("${text}$");
    }
    if span.strong {
        text = format!("**{text}**");
    }
    if span.emphasis {
        text = format!("*{text}*");
    }
    if let Some(link) = &span.link {
        text = format!("[{text}]({link})");
    }

    text
}
