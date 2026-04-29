use app_state::TuiAppState;
use crate::user_command::UserCommand;
use application::events::AppEvent;
use crossterm::event::{Event, EventStream};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{interval, Duration};

pub mod ui;
pub mod app_state;
pub mod app_event_handler;
pub mod input_handler;

pub async fn run(
    mut app: TuiAppState,
    mut event_rx: broadcast::Receiver<AppEvent>,
    command_tx: mpsc::Sender<UserCommand>,
) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut event_stream = EventStream::new();
    let mut tick = interval(Duration::from_millis(250));

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        tokio::select! {
            event = event_stream.next() => {
                if let Some(Ok(Event::Key(key))) = event {
                    input_handler::handle_key(&mut app, key, &command_tx).await;
                }
            }
            app_event = event_rx.recv() => {
                if let Ok(evt) = app_event {
                    app_event_handler::handle(&mut app, evt);
                }
            }
            _ = tick.tick() => {}
        }

        if app.should_quit {
            let _ = command_tx.send(UserCommand::Quit).await;
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
