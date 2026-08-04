use std::io::{self, Stdout};

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use ratatui::{Frame, Terminal, TerminalOptions, Viewport};

const MIN_VIEWPORT_HEIGHT: u16 = 5;
const MAX_VIEWPORT_HEIGHT: u16 = 16;

pub(super) struct SelectionView<'a> {
    pub prompt: &'a str,
    pub choices: &'a [String],
    pub cursor: usize,
    pub selected: Option<&'a [bool]>,
    pub no_color: bool,
}

impl SelectionView<'_> {
    fn is_multiple(&self) -> bool {
        self.selected.is_some()
    }
}

pub(super) type SelectionTerminal = Terminal<CrosstermBackend<Stdout>>;

pub(super) fn open_selection_terminal(choice_count: usize) -> io::Result<SelectionTerminal> {
    let content_height = u16::try_from(choice_count).unwrap_or(u16::MAX);
    let height = content_height
        .saturating_add(3)
        .clamp(MIN_VIEWPORT_HEIGHT, MAX_VIEWPORT_HEIGHT);
    Terminal::with_options(
        CrosstermBackend::new(io::stdout()),
        TerminalOptions {
            viewport: Viewport::Inline(height),
        },
    )
}

pub(super) fn draw_selection(
    terminal: &mut SelectionTerminal,
    view: &SelectionView<'_>,
) -> io::Result<()> {
    terminal.draw(|frame| render_selection(frame, view))?;
    Ok(())
}

fn render_selection(frame: &mut Frame<'_>, view: &SelectionView<'_>) {
    let area = frame.area();
    if area.width < 20 || area.height < MIN_VIEWPORT_HEIGHT {
        frame.render_widget(
            Paragraph::new("Terminal too small — resize to continue")
                .style(Style::default().add_modifier(Modifier::BOLD)),
            area,
        );
        return;
    }

    let accent = if view.no_color {
        Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    };
    let border_style = if view.no_color {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let muted_style = if view.no_color {
        Style::default().add_modifier(Modifier::DIM)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(Line::from(vec![
            Span::styled(" ? ", border_style.add_modifier(Modifier::BOLD)),
            Span::raw(view.prompt),
            Span::raw(" "),
        ]));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [list_area, footer_area] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(inner);
    let items = view
        .choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let marker = view.selected.map_or("", |selected| {
                if selected.get(index).copied().unwrap_or(false) {
                    "◉ "
                } else {
                    "○ "
                }
            });
            ListItem::new(format!("{marker}{choice}"))
        })
        .collect::<Vec<_>>();
    let list = List::new(items)
        .highlight_symbol("❯ ")
        .highlight_style(accent)
        .repeat_highlight_symbol(true);
    let mut state = ListState::default().with_selected(Some(view.cursor));
    frame.render_stateful_widget(list, list_area, &mut state);

    let footer = if area.width < 52 {
        if view.is_multiple() {
            "↑↓ move · Space toggle · Enter done"
        } else {
            "↑↓ move · Enter select · Esc close"
        }
    } else if view.is_multiple() {
        "[↑/k] up  [↓/j] down  [Space] toggle  [Enter] confirm  [Esc/q] cancel"
    } else {
        "[↑/k] up  [↓/j] down  [Enter] select  [Esc/q] cancel"
    };
    frame.render_widget(Paragraph::new(footer).style(muted_style), footer_area);
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::style::{Color, Modifier};

    use super::{SelectionView, render_selection};

    fn render(width: u16, height: u16, view: &SelectionView<'_>) -> Vec<String> {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal
            .draw(|frame| render_selection(frame, view))
            .expect("render selection");
        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect()
    }

    #[test]
    fn renders_discoverable_single_select_at_normal_width() {
        let choices = vec!["Build".into(), "Test".into(), "Deploy".into()];
        let lines = render(
            80,
            8,
            &SelectionView {
                prompt: "Choose an action",
                choices: &choices,
                cursor: 1,
                selected: None,
                no_color: false,
            },
        );
        assert!(lines[0].contains("? Choose an action"));
        assert!(lines.iter().any(|line| line.contains("❯ Test")));
        assert!(lines[6].contains("[Enter] select"));
    }

    #[test]
    fn renders_multiselect_and_compact_footer_without_color() {
        let choices = vec!["Logging".into(), "HTTP".into(), "Config".into()];
        let selected = [true, false, true];
        let view = SelectionView {
            prompt: "Features",
            choices: &choices,
            cursor: 1,
            selected: Some(&selected),
            no_color: true,
        };
        let lines = render(44, 7, &view);
        assert!(lines.iter().any(|line| line.contains("◉ Logging")));
        assert!(lines.iter().any(|line| line.contains("❯ ○ HTTP")));
        assert!(lines[5].contains("Space toggle"));

        let backend = TestBackend::new(44, 7);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal
            .draw(|frame| render_selection(frame, &view))
            .expect("render monochrome selection");
        let content = terminal.backend().buffer().content();
        assert!(
            content
                .iter()
                .all(|cell| cell.fg == Color::Reset && cell.bg == Color::Reset)
        );
        assert!(
            content
                .iter()
                .any(|cell| cell.modifier.contains(Modifier::REVERSED))
        );
    }

    #[test]
    fn shows_a_resize_message_below_the_minimum_width() {
        let choices = vec!["One".into()];
        let lines = render(
            18,
            5,
            &SelectionView {
                prompt: "Pick",
                choices: &choices,
                cursor: 0,
                selected: None,
                no_color: false,
            },
        );
        assert!(lines.join("").contains("Terminal too sma"));
    }
}
