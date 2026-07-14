//! Immediate-mode Ratatui rendering for the Pleiades workspace.

use pleiades_agent_core::conversation::MessageRole;
use pleiades_agent_core::provider::AgentActivityStatus;
use pleiades_agent_engine::Activity;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
};

use crate::markdown::render_markdown;
use crate::state::{AppState, Focus, Overlay, PickerKind};
use crate::theme::Theme;

pub fn render(frame: &mut Frame<'_>, app: &mut AppState) {
    let theme = app.theme;
    frame.render_widget(Block::default().style(theme.base()), frame.area());
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(5),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, rows[0]);
    if rows[1].width >= 100 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
            .split(rows[1]);
        render_conversation(frame, app, columns[0]);
        render_activity(frame, app, columns[1]);
    } else {
        let height = rows[1].height;
        let activity_height = if height >= 18 { 7 } else { 4 };
        let columns = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(4), Constraint::Length(activity_height)])
            .split(rows[1]);
        render_conversation(frame, app, columns[0]);
        render_activity(frame, app, columns[1]);
    }
    render_composer(frame, app, rows[2]);
    render_status(frame, app, rows[3]);

    if let Some(overlay) = app.overlay.clone() {
        render_overlay(frame, app, overlay);
    }
}

fn render_header(frame: &mut Frame<'_>, app: &AppState, area: Rect) {
    let theme = app.theme;
    let mode_style = match app.mode {
        pleiades_agent_engine::AgentMode::Plan => Style::default().fg(theme.info),
        pleiades_agent_engine::AgentMode::Agent => Style::default().fg(theme.success),
        pleiades_agent_engine::AgentMode::Unrestricted => Style::default().fg(theme.warning),
    };
    let status = if app.running { "working" } else { "ready" };
    let header = Line::from(vec![
        Span::styled(format!(" {} PLEIADES ", theme.symbols.agent), theme.title()),
        Span::styled(
            format!("{} / {}", app.provider, app.model),
            Style::default().fg(theme.info),
        ),
        Span::styled("   ", theme.muted()),
        Span::styled(&app.workspace_name, Style::default().fg(theme.foreground)),
        Span::styled("   ", theme.muted()),
        Span::styled(app.mode.label(), mode_style.add_modifier(Modifier::BOLD)),
        Span::styled(format!("   {status}"), theme.muted()),
    ]);
    frame.render_widget(
        Paragraph::new(header).block(panel_block(" Seven Sisters ", theme, false)),
        area,
    );
}

fn render_conversation(frame: &mut Frame<'_>, app: &AppState, area: Rect) {
    let theme = app.theme;
    let mut lines = Vec::new();
    if app.messages.is_empty() {
        lines.extend([
            Line::default(),
            Line::from(Span::styled(
                format!("  {} Describe a coding task to begin", theme.symbols.agent),
                theme.title(),
            )),
            Line::from(Span::styled(
                "  Pleiades will inspect, plan, edit, validate, and review its work.",
                theme.muted(),
            )),
        ]);
    } else {
        let message_offset = app.conversation_scroll as usize / 20;
        let visible_end = app.messages.len().saturating_sub(message_offset);
        let visible_start = visible_end.saturating_sub(24);
        if visible_start > 0 {
            lines.push(Line::from(Span::styled(
                format!(
                    "  {} {} earlier messages · continue scrolling",
                    theme.symbols.context, visible_start
                ),
                theme.muted(),
            )));
            lines.push(Line::default());
        }
        for message in &app.messages[visible_start..visible_end] {
            let (label, symbol, style) = match message.role {
                MessageRole::User => (
                    "YOU",
                    theme.symbols.context,
                    Style::default().fg(theme.starlight),
                ),
                MessageRole::Assistant => ("PLEIADES", theme.symbols.agent, theme.title()),
                MessageRole::System => (
                    "SYSTEM",
                    theme.symbols.suggestion,
                    Style::default().fg(theme.info),
                ),
                MessageRole::Tool => ("TOOL", theme.symbols.tool, Style::default().fg(theme.info)),
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {symbol} {label}"),
                    style.add_modifier(Modifier::BOLD),
                ),
                if message.streaming {
                    Span::styled("  streaming", theme.muted())
                } else {
                    Span::raw("")
                },
            ]));
            if let Some(reasoning) = &message.reasoning {
                if !reasoning.is_empty() {
                    lines.push(Line::from(Span::styled(
                        format!(
                            "   {} {}",
                            theme.symbols.suggestion,
                            compact(reasoning, 180)
                        ),
                        theme.muted().add_modifier(Modifier::ITALIC),
                    )));
                }
            }
            lines.extend(
                render_markdown(&message.content, theme)
                    .into_iter()
                    .map(indent_line),
            );
            lines.push(Line::default());
        }
    }

    let inner_height = area.height.saturating_sub(2) as usize;
    let top = lines
        .len()
        .saturating_sub(inner_height)
        .saturating_sub(app.conversation_scroll as usize % 20) as u16;
    let block = panel_block(" Conversation ", theme, app.focus == Focus::Conversation);
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((top, 0)),
        area,
    );
}

fn render_activity(frame: &mut Frame<'_>, app: &mut AppState, area: Rect) {
    let theme = app.theme;
    let items: Vec<ListItem<'static>> = if app.activities.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            " · No activity yet",
            theme.muted(),
        )))]
    } else {
        app.activities
            .iter()
            .map(|activity| ListItem::new(activity_line(activity, theme)))
            .collect()
    };
    let mut state = ListState::default();
    if !app.activities.is_empty() {
        app.selected_activity = app.selected_activity.min(app.activities.len() - 1);
        state.select(Some(app.selected_activity));
    }
    let list = List::new(items)
        .block(panel_block(
            " Activity ",
            theme,
            app.focus == Focus::Activity,
        ))
        .highlight_style(Style::default().bg(theme.surface_alt))
        .highlight_symbol("▸ ");
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_composer(frame: &mut Frame<'_>, app: &AppState, area: Rect) {
    let theme = app.theme;
    let title = if app.running {
        format!(" Queue follow-up  ·  {} queued ", app.queued)
    } else {
        " Ask Pleiades  ·  Enter send  Alt+Enter newline ".to_string()
    };
    let block = panel_block(&title, theme, app.focus == Focus::Composer).style(theme.base());
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(&app.composer, inner);
}

fn render_status(frame: &mut Frame<'_>, app: &AppState, area: Rect) {
    let theme = app.theme;
    let branch = app.branch.as_deref().unwrap_or("no-git");
    let dirty = if app.dirty { "*" } else { "" };
    let tokens = app
        .usage
        .as_ref()
        .map(|usage| format!("{} tokens", usage.input_tokens + usage.output_tokens))
        .unwrap_or_else(|| "— tokens".to_string());
    let elapsed = app.elapsed();
    let running = app
        .active_activity()
        .map(|activity| compact(&activity.title, 28))
        .unwrap_or_else(|| "idle".into());
    let line = Line::from(vec![
        Span::styled(
            format!(" {} {} ", app.mode.label(), theme.symbols.context),
            Style::default()
                .fg(theme.background)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                " {branch}{dirty}  {tokens}  {:02}:{:02}  {running} ",
                elapsed.as_secs() / 60,
                elapsed.as_secs() % 60
            ),
            Style::default().fg(theme.foreground).bg(theme.surface_alt),
        ),
        Span::styled("  Ctrl+P palette  F1 help  Ctrl+C cancel ", theme.muted()),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_overlay(frame: &mut Frame<'_>, app: &AppState, overlay: Overlay) {
    let theme = app.theme;
    let area = centered_rect(78, 72, frame.area());
    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::default().style(Style::default().bg(theme.surface)),
        area,
    );
    match overlay {
        Overlay::Permission(request) => {
            let text = vec![
                Line::from(Span::styled(
                    format!("{} Permission required", theme.symbols.paused),
                    theme.title(),
                )),
                Line::default(),
                field("Operation", &request.operation, theme),
                field("Target", &request.target, theme),
                field("Reason", &request.reason, theme),
                field("Risk", &request.risk, theme),
                Line::default(),
                Line::from(vec![
                    Span::styled(
                        " [a] Allow once ",
                        Style::default().fg(theme.background).bg(theme.success),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        " [s] Allow session ",
                        Style::default().fg(theme.background).bg(theme.info),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        " [d] Deny once  ",
                        Style::default().fg(theme.background).bg(theme.warning),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        " [x] Deny session  ",
                        Style::default().fg(theme.foreground).bg(theme.error),
                    ),
                ]),
            ];
            render_modal(frame, area, " Safe autonomy ", text, theme);
        }
        Overlay::Help { query } => {
            let mut text = vec![field("Search", &query, theme), Line::default()];
            let commands = app.palette_listing(&query);
            if commands.is_empty() {
                text.push(Line::from(Span::styled(
                    "No matching commands.",
                    theme.muted(),
                )));
            } else {
                text.extend(
                    commands
                        .into_iter()
                        .map(|(command, description)| field(&command, &description, theme)),
                );
            }
            render_modal(
                frame,
                area,
                " Searchable help  ·  type to filter  Esc close ",
                text,
                theme,
            );
        }
        Overlay::CommandPalette { selected, query } => {
            let matches = app.palette_listing(&query);
            let mut text = vec![field("Search", &query, theme), Line::default()];
            text.extend(matches.iter().enumerate().map(
                |(visible_index, (command, description))| {
                    let style = if visible_index == selected {
                        Style::default()
                            .fg(theme.background)
                            .bg(theme.primary)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.foreground)
                    };
                    Line::from(Span::styled(
                        format!(
                            " {} {command:<24} {description}",
                            if visible_index == selected {
                                "›"
                            } else {
                                " "
                            }
                        ),
                        style,
                    ))
                },
            ));
            render_modal(
                frame,
                area,
                " Command palette  ·  ↑↓ select  Enter run ",
                text,
                theme,
            );
        }
        Overlay::Picker {
            kind,
            selected,
            query,
        } => {
            let options = app.filtered_picker_options(kind, &query);
            let mut text = vec![field("Search", &query, theme), Line::default()];
            if options.is_empty() {
                text.push(Line::from(Span::styled(
                    "No matching entries.",
                    theme.muted(),
                )));
            } else {
                text.extend(options.iter().enumerate().map(|(index, value)| {
                    let style = if index == selected {
                        Style::default()
                            .fg(theme.background)
                            .bg(theme.primary)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.foreground)
                    };
                    Line::from(Span::styled(
                        format!(" {} {value}", if index == selected { "›" } else { " " }),
                        style,
                    ))
                }));
            }
            render_modal(
                frame,
                area,
                &format!(
                    " {} picker  ·  type to filter  Enter select ",
                    picker_label(kind)
                ),
                text,
                theme,
            );
        }
        Overlay::Diff => {
            let lines = if app.diff.trim().is_empty() {
                vec![Line::from(Span::styled(
                    "No workspace changes detected.",
                    theme.muted(),
                ))]
            } else {
                app.diff
                    .lines()
                    .map(|line| {
                        let style = if line.starts_with('+') && !line.starts_with("+++") {
                            Style::default().fg(theme.diff_add)
                        } else if line.starts_with('-') && !line.starts_with("---") {
                            Style::default().fg(theme.diff_remove)
                        } else if line.starts_with("@@") {
                            Style::default().fg(theme.info)
                        } else {
                            Style::default().fg(theme.foreground)
                        };
                        Line::from(Span::styled(line.to_string(), style))
                    })
                    .collect()
            };
            render_modal(frame, area, " Diff review  ·  Esc close ", lines, theme);
        }
        Overlay::ToolOutput { activity_id } => {
            let output = app
                .outputs
                .get(&activity_id)
                .map(String::as_str)
                .unwrap_or("No captured output for this activity.");
            let text = output
                .lines()
                .map(|line| {
                    Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(theme.foreground),
                    ))
                })
                .collect();
            render_modal(
                frame,
                area,
                &format!(" Tool output · {activity_id} "),
                text,
                theme,
            );
        }
        Overlay::ToolDetails { activity_id } => {
            let mut text = Vec::new();
            if let Some(activity) = app.activities.iter().find(|item| item.id == activity_id) {
                text.extend([
                    field("Activity", &activity.title, theme),
                    field("Kind", &activity.kind.to_string(), theme),
                    field("Status", &activity.status.to_string(), theme),
                    field(
                        "Duration",
                        &activity
                            .duration_ms
                            .map(|value| format!("{value} ms"))
                            .unwrap_or_else(|| "running".to_string()),
                        theme,
                    ),
                    Line::default(),
                ]);
                if let Some(detail) = &activity.detail {
                    text.extend(detail.lines().map(|line| Line::from(line.to_string())));
                }
                if let Some(output) = app.outputs.get(&activity_id) {
                    text.push(Line::default());
                    text.push(Line::from(Span::styled("Captured output", theme.title())));
                    text.extend(output.lines().map(|line| Line::from(line.to_string())));
                }
            } else {
                text.push(Line::from(Span::styled(
                    "Activity is no longer available.",
                    theme.muted(),
                )));
            }
            render_modal(frame, area, " Tool details  ·  Esc close ", text, theme);
        }
        Overlay::Configuration => {
            let text = vec![
                field("Theme", app.theme.name, theme),
                field("Provider", &app.provider, theme),
                field("Model", &app.model, theme),
                field("Mode", app.mode.label(), theme),
                field("Workspace", &app.workspace.display().to_string(), theme),
                Line::default(),
                Line::from(Span::styled("Keyboard configuration", theme.title())),
                field("Ctrl+R", "select provider", theme),
                field("Ctrl+M", "select model", theme),
                field("Ctrl+F", "find workspace file", theme),
                field("Ctrl+L", "open saved session", theme),
                field("/mode", "plan | agent | unrestricted", theme),
                Line::default(),
                Line::from(Span::styled(
                    "Persistent values can be changed with `pleiades config set`.",
                    theme.muted(),
                )),
            ];
            render_modal(frame, area, " Configuration  ·  Esc close ", text, theme);
        }
        Overlay::Diagnostics => {
            let text = vec![
                field("Workspace", &app.workspace.display().to_string(), theme),
                field("Session", &app.session_id, theme),
                field("Provider", &app.provider, theme),
                field("Model", &app.model, theme),
                field("Mode", app.mode.label(), theme),
                field(
                    "Git",
                    &format!(
                        "{}{}",
                        app.branch.as_deref().unwrap_or("not detected"),
                        if app.dirty { " (dirty)" } else { "" }
                    ),
                    theme,
                ),
                field("Status", &app.status, theme),
            ];
            render_modal(frame, area, " Diagnostics  ·  Esc close ", text, theme);
        }
        Overlay::Document(document) => {
            let mut text = Vec::new();
            for section in document.sections {
                text.push(Line::from(Span::styled(section.heading, theme.title())));
                text.extend(section.body.lines().map(|line| {
                    Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(theme.foreground),
                    ))
                }));
                text.push(Line::default());
            }
            if text.is_empty() {
                text.push(Line::from(Span::styled(
                    "No details available.",
                    theme.muted(),
                )));
            }
            render_modal(
                frame,
                area,
                &format!(" {}  ·  Esc close ", document.title),
                text,
                theme,
            );
        }
    }
}

fn activity_line(activity: &Activity, theme: Theme) -> Line<'static> {
    let (symbol, style) = match activity.status {
        AgentActivityStatus::Queued => (theme.symbols.context, theme.muted()),
        AgentActivityStatus::Running => (theme.symbols.running, Style::default().fg(theme.info)),
        AgentActivityStatus::WaitingForApproval => (
            theme.symbols.paused,
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        AgentActivityStatus::Completed => {
            (theme.symbols.success, Style::default().fg(theme.success))
        }
        AgentActivityStatus::Failed => (
            theme.symbols.failure,
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        AgentActivityStatus::Cancelled => (theme.symbols.paused, theme.muted()),
    };
    let duration = activity
        .duration_ms
        .map(|ms| format!(" {ms}ms"))
        .unwrap_or_default();
    Line::from(vec![
        Span::styled(format!("{symbol} "), style),
        Span::styled(
            compact(&activity.title, 52),
            if activity.status == AgentActivityStatus::Completed {
                theme.muted()
            } else {
                style
            },
        ),
        Span::styled(duration, theme.muted()),
    ])
}

fn render_modal(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    lines: Vec<Line<'static>>,
    theme: Theme,
) {
    let block = panel_block(title, theme, true).style(Style::default().bg(theme.surface));
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn field(label: &str, value: &str, theme: Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:>11}  "), theme.muted()),
        Span::styled(value.to_string(), Style::default().fg(theme.foreground)),
    ])
}

fn picker_label(kind: PickerKind) -> &'static str {
    match kind {
        PickerKind::Provider => "Provider",
        PickerKind::Model => "Model",
        PickerKind::File => "File",
        PickerKind::Session => "Session",
    }
}

fn panel_block<'a>(title: &'a str, theme: Theme, focused: bool) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(if theme.name == "ascii" {
            BorderType::Plain
        } else {
            BorderType::Rounded
        })
        .border_style(if focused {
            theme.focused_border()
        } else {
            theme.border()
        })
        .title(Span::styled(
            title,
            if focused {
                theme.title()
            } else {
                theme.muted()
            },
        ))
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn indent_line(line: Line<'static>) -> Line<'static> {
    let mut spans = vec![Span::raw("   ")];
    spans.extend(line.spans);
    Line::from(spans)
}

fn compact(value: &str, max_chars: usize) -> String {
    let value = value.replace(['\n', '\r'], " ");
    if value.chars().count() <= max_chars {
        return value;
    }
    value
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>()
        + "…"
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::{state::AppState, theme::Theme};
    use pleiades_agent_engine::AgentMode;
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;

    #[test]
    fn renders_all_persistent_regions() {
        let backend = TestBackend::new(120, 32);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = AppState::new(
            Theme::default(),
            PathBuf::from("/work/pleiades"),
            "mock".into(),
            "mock-1".into(),
            AgentMode::Agent,
        );
        terminal.draw(|frame| render(frame, &mut app)).unwrap();
        let content = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("PLEIADES"));
        assert!(content.contains("Conversation"));
        assert!(content.contains("Activity"));
        assert!(content.contains("Ask Pleiades"));
    }

    #[test]
    fn seven_sisters_shell_snapshot() {
        let backend = TestBackend::new(80, 22);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = AppState::new(
            Theme::default(),
            PathBuf::from("/work/pleiades"),
            "openai-subscription".into(),
            "codex-default".into(),
            AgentMode::Agent,
        );
        terminal.draw(|frame| render(frame, &mut app)).unwrap();
        let buffer = terminal.backend().buffer();
        let mut snapshot = String::new();
        for y in 0..buffer.area.height {
            let row = (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>();
            snapshot.push_str(row.trim_end());
            snapshot.push('\n');
        }
        insta::assert_snapshot!(snapshot);
    }

    #[test]
    fn renders_across_small_and_large_terminal_sizes() {
        for (width, height) in [(32, 10), (80, 24), (180, 50)] {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).unwrap();
            let mut app = AppState::new(
                Theme::default(),
                PathBuf::from("/work/pleiades"),
                "mock".into(),
                "mock-1".into(),
                AgentMode::Agent,
            );
            terminal.draw(|frame| render(frame, &mut app)).unwrap();
            assert_eq!(terminal.backend().buffer().area.width, width);
            assert_eq!(terminal.backend().buffer().area.height, height);
        }
    }
}
