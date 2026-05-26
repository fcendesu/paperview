#[must_use]
pub fn inline_text(source: &str) -> String {
    format!("${}$", inline_source(source))
}

#[must_use]
pub fn inline_source(source: &str) -> String {
    source.trim().to_owned()
}

#[must_use]
pub fn display_source(source: &str) -> String {
    source.trim().to_owned()
}

#[must_use]
pub fn readable_preview(source: &str) -> Option<String> {
    let mut output = display_source(source);

    if output.is_empty() {
        return None;
    }

    output = replace_frac(&output);
    output = replace_group_command(&output, "\\sqrt", "√");
    output = replace_group_command(&output, "\\vec", "vec");
    output = output.replace("\\leftarrow", "←");
    output = output.replace("\\rightarrow", "→");
    output = output.replace("\\leftrightarrow", "↔");
    output = output.replace("\\left", "");
    output = output.replace("\\right", "");

    for (from, to) in SYMBOLS {
        output = output.replace(from, to);
    }

    output = output.replace("\\,", "");
    output = replace_script(&output, '^', superscript_character);
    output = replace_script(&output, '_', subscript_character);
    output = output.replace(['{', '}'], "");
    output = output.split_whitespace().collect::<Vec<_>>().join(" ");

    (output != display_source(source)).then_some(output)
}

fn replace_frac(source: &str) -> String {
    let mut output = source.to_owned();

    while let Some(start) = output.find("\\frac{") {
        let numerator_start = start + "\\frac".len();
        let Some((numerator, after_numerator)) = braced_group(&output, numerator_start) else {
            break;
        };
        let Some((denominator, after_denominator)) = braced_group(&output, after_numerator) else {
            break;
        };

        output.replace_range(
            start..after_denominator,
            &format!("({numerator}) / ({denominator})"),
        );
    }

    output
}

fn replace_group_command(source: &str, command: &str, replacement: &str) -> String {
    let mut output = source.to_owned();
    let pattern = format!("{command}{{");

    while let Some(start) = output.find(&pattern) {
        let group_start = start + command.len();
        let Some((group, after_group)) = braced_group(&output, group_start) else {
            break;
        };

        output.replace_range(start..after_group, &format!("{replacement}({group})"));
    }

    output
}

fn braced_group(source: &str, start: usize) -> Option<(String, usize)> {
    let mut chars = source[start..].char_indices();

    if chars.next()?.1 != '{' {
        return None;
    }

    let mut depth = 1usize;
    let content_start = start + 1;

    for (offset, character) in chars {
        match character {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let end = start + offset;
                    return Some((source[content_start..end].to_owned(), end + 1));
                }
            }
            _ => {}
        }
    }

    None
}

fn replace_script(source: &str, marker: char, convert: fn(char) -> Option<char>) -> String {
    let mut output = String::new();
    let mut chars = source.chars().peekable();

    while let Some(character) = chars.next() {
        if character != marker {
            output.push(character);
            continue;
        }

        if chars.peek() == Some(&'{') {
            chars.next();
            let mut converted = String::new();
            let mut converted_all = true;

            for script_character in chars.by_ref() {
                if script_character == '}' {
                    break;
                }

                if let Some(character) = convert(script_character) {
                    converted.push(character);
                } else {
                    converted_all = false;
                    converted.push(script_character);
                }
            }

            if converted_all && !converted.is_empty() {
                output.push_str(&converted);
            } else {
                output.push(marker);
                output.push('{');
                output.push_str(&converted);
                output.push('}');
            }
            continue;
        }

        if let Some(next) = chars.peek().copied()
            && let Some(converted) = convert(next)
        {
            output.push(converted);
            chars.next();
        } else {
            output.push(character);
        }
    }

    output
}

fn superscript_character(character: char) -> Option<char> {
    match character {
        '0' => Some('⁰'),
        '1' => Some('¹'),
        '2' => Some('²'),
        '3' => Some('³'),
        '4' => Some('⁴'),
        '5' => Some('⁵'),
        '6' => Some('⁶'),
        '7' => Some('⁷'),
        '8' => Some('⁸'),
        '9' => Some('⁹'),
        '+' => Some('⁺'),
        '-' => Some('⁻'),
        '=' => Some('⁼'),
        '(' => Some('⁽'),
        ')' => Some('⁾'),
        'n' => Some('ⁿ'),
        'i' => Some('ⁱ'),
        _ => None,
    }
}

fn subscript_character(character: char) -> Option<char> {
    match character {
        '0' => Some('₀'),
        '1' => Some('₁'),
        '2' => Some('₂'),
        '3' => Some('₃'),
        '4' => Some('₄'),
        '5' => Some('₅'),
        '6' => Some('₆'),
        '7' => Some('₇'),
        '8' => Some('₈'),
        '9' => Some('₉'),
        '+' => Some('₊'),
        '-' => Some('₋'),
        '=' => Some('₌'),
        '(' => Some('₍'),
        ')' => Some('₎'),
        'a' => Some('ₐ'),
        'e' => Some('ₑ'),
        'h' => Some('ₕ'),
        'i' => Some('ᵢ'),
        'j' => Some('ⱼ'),
        'k' => Some('ₖ'),
        'l' => Some('ₗ'),
        'm' => Some('ₘ'),
        'n' => Some('ₙ'),
        'o' => Some('ₒ'),
        'p' => Some('ₚ'),
        'r' => Some('ᵣ'),
        's' => Some('ₛ'),
        't' => Some('ₜ'),
        'u' => Some('ᵤ'),
        'v' => Some('ᵥ'),
        'x' => Some('ₓ'),
        _ => None,
    }
}

const SYMBOLS: &[(&str, &str)] = &[
    ("\\varepsilon", "ε"),
    ("\\vartheta", "ϑ"),
    ("\\varphi", "φ"),
    ("\\Gamma", "Γ"),
    ("\\Delta", "Δ"),
    ("\\Theta", "Θ"),
    ("\\Lambda", "Λ"),
    ("\\Pi", "Π"),
    ("\\Sigma", "Σ"),
    ("\\Phi", "Φ"),
    ("\\Omega", "Ω"),
    ("\\alpha", "α"),
    ("\\beta", "β"),
    ("\\gamma", "γ"),
    ("\\delta", "δ"),
    ("\\epsilon", "ε"),
    ("\\theta", "θ"),
    ("\\lambda", "λ"),
    ("\\mu", "μ"),
    ("\\pi", "π"),
    ("\\sigma", "σ"),
    ("\\phi", "φ"),
    ("\\omega", "ω"),
    ("\\partial", "∂"),
    ("\\nabla", "∇"),
    ("\\times", "×"),
    ("\\cdot", "·"),
    ("\\div", "÷"),
    ("\\pm", "±"),
    ("\\mp", "∓"),
    ("\\leq", "≤"),
    ("\\le", "≤"),
    ("\\geq", "≥"),
    ("\\ge", "≥"),
    ("\\neq", "≠"),
    ("\\ne", "≠"),
    ("\\approx", "≈"),
    ("\\equiv", "≡"),
    ("\\propto", "∝"),
    ("\\cup", "∪"),
    ("\\cap", "∩"),
    ("\\sum", "Σ"),
    ("\\prod", "Π"),
    ("\\int", "∫"),
    ("\\oint", "∮"),
    ("\\forall", "∀"),
    ("\\exists", "∃"),
    ("\\infty", "∞"),
    ("\\notin", "∉"),
    ("\\subseteq", "⊆"),
    ("\\subset", "⊂"),
    ("\\in", "∈"),
    ("\\lim", "lim"),
    ("\\sin", "sin"),
    ("\\cos", "cos"),
    ("\\tan", "tan"),
    ("\\log", "log"),
    ("\\ln", "ln"),
    ("\\Rightarrow", "⇒"),
    ("\\Leftarrow", "⇐"),
    ("\\Leftrightarrow", "⇔"),
    ("\\leftarrow", "←"),
    ("\\leftrightarrow", "↔"),
    ("\\rightarrow", "→"),
    ("\\mapsto", "↦"),
    ("\\to", "→"),
];

#[cfg(test)]
mod tests {
    use super::readable_preview;

    #[test]
    fn renders_readable_preview_for_common_math_tokens() {
        assert_eq!(
            readable_preview(r"\frac{\alpha_1}{x^2} + \sqrt{y} \to \infty"),
            Some("(α₁) / (x²) + √(y) → ∞".to_owned())
        );
    }

    #[test]
    fn renders_readable_preview_for_academic_notation() {
        assert_eq!(
            readable_preview(r"\left( \sum_{i=1}^{10} x_i \right) \leq \int_0^\infty f(t) \, dt"),
            Some("( Σᵢ₌₁¹⁰ xᵢ ) ≤ ∫₀^∞ f(t) dt".to_owned())
        );
        assert_eq!(
            readable_preview(r"\nabla \cdot \vec{F} \Rightarrow \Delta x \notin A \subseteq B"),
            Some("∇ · vec(F) ⇒ Δ x ∉ A ⊆ B".to_owned())
        );
    }

    #[test]
    fn omits_preview_when_source_is_already_plain() {
        assert_eq!(readable_preview("E = mc"), None);
    }
}
