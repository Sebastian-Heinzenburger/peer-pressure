use crate::tui::app_state::{TuiAppState, InputMode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &TuiAppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(chunks[0]);

    render_peer_list(frame, app, main_chunks[0]);
    render_chat(frame, app, main_chunks[1]);
    render_input(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);
}

fn render_peer_list(frame: &mut Frame, app: &TuiAppState, area: Rect) {
    let items: Vec<ListItem> = app
        .peers
        .iter()
        .enumerate()
        .map(|(i, peer)| {
            let connected = app.connected_peers.contains(peer);
            let indicator = if connected { "●" } else { "○" };
            let style = if i == app.selected_peer {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if connected {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", indicator), style),
                Span::styled(peer.to_string(), style),
            ]))
        })
        .collect();

    let peer_list = List::new(items).block(
        Block::default()
            .title(" Peers ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    );

    frame.render_widget(peer_list, area);
}

fn render_chat(frame: &mut Frame, app: &TuiAppState, area: Rect) {
    let title = app
        .selected_peer_id()
        .map(|p| format!(" Chat with {} ", p))
        .unwrap_or_else(|| " No peer selected ".to_string());

    let messages = app.current_messages();
    let lines: Vec<Line> = messages
        .iter()
        .map(|msg| {
            if msg.sent_by_me {
                let indicator = if msg.delivered { " ✓" } else { " !" };
                let indicator_color = if msg.delivered { Color::Green } else { Color::Red };
                Line::from(vec![
                    Span::styled("You: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&msg.content),
                    Span::styled(indicator, Style::default().fg(indicator_color)),
                ])
            } else {
                Line::from(vec![
                    Span::styled("Peer: ", Style::default().fg(Color::Green)),
                    Span::raw(&msg.content),
                ])
            }
        })
        .collect();

    let chat_widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(chat_widget, area);
}

fn render_input(frame: &mut Frame, app: &TuiAppState, area: Rect) {
    let (title, style) = match app.input_mode {
        InputMode::Normal => (
            " Press 'i' to type, 'q' to quit ",
            Style::default().fg(Color::DarkGray),
        ),
        InputMode::Editing => (
            " Type message (Enter=send, /add <ip>, /connect, Esc=cancel) ",
            Style::default().fg(Color::Yellow),
        ),
    };

    let input = Paragraph::new(app.input.as_str())
        .style(style)
        .block(Block::default().title(title).borders(Borders::ALL));

    frame.render_widget(input, area);

    if app.input_mode == InputMode::Editing {
        frame.set_cursor_position((area.x + app.input.len() as u16 + 1, area.y + 1));
    }
}

fn render_status_bar(frame: &mut Frame, app: &TuiAppState, area: Rect) {
    let connected_count = app.connected_peers.len();
    let total_count = app.peers.len();
    let status = format!(
        " Peers: {}/{} connected | Tab: switch peer | /add <ip>: add peer | /connect: connect to selected peer ",
        connected_count, total_count
    );
    let status_bar = Paragraph::new(status).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(status_bar, area);
}
