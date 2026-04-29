use crate::tui::app_state::{InputMode, TuiAppState};
use crate::user_command::UserCommand;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

pub async fn handle_key(
    app: &mut TuiAppState,
    key: KeyEvent,
    command_tx: &mpsc::Sender<UserCommand>,
) {
    match app.input_mode {
        InputMode::Normal => handle_normal_mode(app, key, command_tx).await,
        InputMode::Editing => handle_editing_mode(app, key, command_tx).await,
    }
}

async fn handle_normal_mode(
    app: &mut TuiAppState,
    key: KeyEvent,
    _command_tx: &mpsc::Sender<UserCommand>,
) {
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Char('i') | KeyCode::Enter => {
            app.input_mode = InputMode::Editing;
        }
        KeyCode::Tab | KeyCode::Down if !app.peers.is_empty() => {
            app.selected_peer = (app.selected_peer + 1) % app.peers.len();
        }
        KeyCode::BackTab | KeyCode::Up if !app.peers.is_empty() => {
            app.selected_peer = app
                .selected_peer
                .checked_sub(1)
                .unwrap_or(app.peers.len() - 1);
        }
        _ => {}
    }
}

async fn handle_editing_mode(
    app: &mut TuiAppState,
    key: KeyEvent,
    command_tx: &mpsc::Sender<UserCommand>,
) {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.input.clear();
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Enter => {
            let input = app.input.drain(..).collect::<String>();
            if input.is_empty() {
                return;
            }

            if let Some(address) = input.strip_prefix("/add ") {
                let address = address.trim().to_string();
                if !address.is_empty() {
                    let _ = command_tx.send(UserCommand::AddPeer { address }).await;
                }
            } else if input.trim() == "/connect" {
                if let Some(peer) = app.selected_peer_id().cloned() {
                    let _ = command_tx.send(UserCommand::ConnectToPeer { peer }).await;
                }
            } else if let Some(peer) = app.selected_peer_id().cloned() {
                let _ = command_tx
                    .send(UserCommand::SendMessage { peer, text: input })
                    .await;
            }
        }
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Char(c) => {
            app.input.push(c);
        }
        _ => {}
    }
}
