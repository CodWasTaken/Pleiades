//! Markdown-to-Ratatui rendering.
//!
//! The full-screen UI keeps semantic text instead of producing ANSI bytes so
//! content can be wrapped, scrolled, cached, and redrawn safely.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;

use once_cell::sync::Lazy;

use crate::theme::Theme;

static SYNTAXES: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static SYNTAX_THEMES: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

pub fn render_markdown(source: &str, theme: Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut code_language: Option<String> = None;
    let mut highlighter: Option<HighlightLines<'static>> = None;

    for raw in source.lines() {
        if let Some(language) = raw.strip_prefix("```") {
            if code_language.is_some() {
                code_language = None;
                highlighter = None;
            } else {
                code_language = Some(language.trim().to_string());
                let syntax = SYNTAXES
                    .find_syntax_by_token(language.trim())
                    .unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());
                highlighter = Some(HighlightLines::new(
                    syntax,
                    &SYNTAX_THEMES.themes["base16-ocean.dark"],
                ));
                let label = if language.trim().is_empty() {
                    "code".to_string()
                } else {
                    language.trim().to_string()
                };
                lines.push(Line::from(Span::styled(
                    format!(" {label} "),
                    Style::default().fg(theme.info).bg(theme.surface_alt),
                )));
            }
            continue;
        }

        if code_language.is_some() {
            let mut spans = vec![Span::styled("  ", Style::default().bg(theme.surface))];
            if let Some(highlighter) = highlighter.as_mut() {
                match highlighter.highlight_line(raw, &SYNTAXES) {
                    Ok(regions) => spans.extend(regions.into_iter().map(|(style, text)| {
                        let mut output = Style::default()
                            .fg(ratatui::style::Color::Rgb(
                                style.foreground.r,
                                style.foreground.g,
                                style.foreground.b,
                            ))
                            .bg(theme.surface);
                        if style.font_style.contains(FontStyle::BOLD) {
                            output = output.add_modifier(Modifier::BOLD);
                        }
                        if style.font_style.contains(FontStyle::ITALIC) {
                            output = output.add_modifier(Modifier::ITALIC);
                        }
                        if style.font_style.contains(FontStyle::UNDERLINE) {
                            output = output.add_modifier(Modifier::UNDERLINED);
                        }
                        Span::styled(text.to_string(), output)
                    })),
                    Err(_) => spans.push(Span::styled(
                        raw.to_string(),
                        Style::default().fg(theme.foreground).bg(theme.surface),
                    )),
                }
            }
            lines.push(Line::from(spans));
            continue;
        }

        if let Some(heading) = raw.strip_prefix("### ") {
            lines.push(Line::from(Span::styled(
                heading.to_string(),
                Style::default()
                    .fg(theme.starlight)
                    .add_modifier(Modifier::BOLD),
            )));
        } else if let Some(heading) = raw.strip_prefix("## ") {
            lines.push(Line::from(Span::styled(
                heading.to_string(),
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )));
        } else if let Some(heading) = raw.strip_prefix("# ") {
            lines.push(Line::from(Span::styled(
                heading.to_string(),
                theme.title().add_modifier(Modifier::UNDERLINED),
            )));
        } else if let Some(item) = raw.strip_prefix("- ").or_else(|| raw.strip_prefix("* ")) {
            let mut spans = vec![Span::styled("  • ", Style::default().fg(theme.primary))];
            spans.extend(inline_spans(item, theme));
            lines.push(Line::from(spans));
        } else if let Some(quote) = raw.strip_prefix("> ") {
            let mut spans = vec![Span::styled("│ ", Style::default().fg(theme.border))];
            spans.extend(inline_spans(quote, theme).into_iter().map(|span| {
                Span::styled(
                    span.content.into_owned(),
                    span.style.add_modifier(Modifier::ITALIC),
                )
            }));
            lines.push(Line::from(spans));
        } else if raw.starts_with('|') && raw.ends_with('|') {
            let cells = raw.trim_matches('|').split('|').map(str::trim);
            let mut spans = Vec::new();
            spans.push(Span::styled("│ ", Style::default().fg(theme.border)));
            for cell in cells {
                spans.extend(inline_spans(cell, theme));
                spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
            }
            lines.push(Line::from(spans));
        } else {
            lines.push(Line::from(inline_spans(raw, theme)));
        }
    }

    if source.is_empty() || source.ends_with('\n') {
        lines.push(Line::default());
    }
    lines
}

fn inline_spans(source: &str, theme: Theme) -> Vec<Span<'static>> {
    let mut result = Vec::new();
    let mut rest = source;
    while !rest.is_empty() {
        let Some((index, marker)) = next_marker(rest) else {
            result.push(Span::styled(
                rest.to_string(),
                Style::default().fg(theme.foreground),
            ));
            break;
        };
        if index > 0 {
            result.push(Span::styled(
                rest[..index].to_string(),
                Style::default().fg(theme.foreground),
            ));
        }
        rest = &rest[index..];
        match marker {
            "`" => {
                if let Some(end) = rest[1..].find('`') {
                    result.push(Span::styled(
                        rest[1..end + 1].to_string(),
                        Style::default().fg(theme.starlight).bg(theme.surface_alt),
                    ));
                    rest = &rest[end + 2..];
                } else {
                    result.push(Span::raw("`"));
                    rest = &rest[1..];
                }
            }
            "**" => {
                if let Some(end) = rest[2..].find("**") {
                    result.push(Span::styled(
                        rest[2..end + 2].to_string(),
                        Style::default()
                            .fg(theme.foreground)
                            .add_modifier(Modifier::BOLD),
                    ));
                    rest = &rest[end + 4..];
                } else {
                    result.push(Span::raw("**"));
                    rest = &rest[2..];
                }
            }
            "[" => {
                if let Some(close) = rest.find("](") {
                    if let Some(end) = rest[close + 2..].find(')') {
                        let label = &rest[1..close];
                        let url = &rest[close + 2..close + 2 + end];
                        result.push(Span::styled(
                            format!("{label} ({url})"),
                            Style::default()
                                .fg(theme.info)
                                .add_modifier(Modifier::UNDERLINED),
                        ));
                        rest = &rest[close + end + 3..];
                    } else {
                        result.push(Span::raw("["));
                        rest = &rest[1..];
                    }
                } else {
                    result.push(Span::raw("["));
                    rest = &rest[1..];
                }
            }
            _ => unreachable!(),
        }
    }
    if result.is_empty() {
        result.push(Span::raw(String::new()));
    }
    result
}

fn next_marker(source: &str) -> Option<(usize, &'static str)> {
    [
        (source.find("**"), "**"),
        (source.find('`'), "`"),
        (source.find('['), "["),
    ]
    .into_iter()
    .filter_map(|(index, marker)| index.map(|index| (index, marker)))
    .min_by_key(|(index, _)| *index)
}

#[cfg(test)]
mod tests {
    use super::render_markdown;
    use crate::theme::Theme;

    #[test]
    fn produces_semantic_ratatui_lines() {
        let lines = render_markdown(
            "# Title\n- use `cargo test`\n[docs](https://example.com)",
            Theme::default(),
        );
        assert_eq!(lines.len(), 3);
        assert!(lines[1].spans.len() >= 3);
    }

    #[test]
    fn syntax_highlights_fenced_rust_as_native_spans() {
        let lines = render_markdown(
            "```rust\nfn main() { let answer = 42; }\n```",
            Theme::default(),
        );
        assert!(lines[1].spans.len() > 3);
        let colors = lines[1]
            .spans
            .iter()
            .filter_map(|span| span.style.fg)
            .collect::<std::collections::HashSet<_>>();
        assert!(colors.len() > 1);
    }
}
