use application::ports::event_receiver::EventReceiverFactory;
use application::ports::event_sender::EventSender;
use application::ports::inbound_message_handler::InboundMessageHandler;
use application::ports::repository::chat::ChatRepository;
use application::ports::repository::peer::PeerRepository;
use application::ports::sender_service::MessageSenderService;
use application::use_cases::{AddPeer, ConnectAndResend, ReceiveMessage, SendMessage};
use clap::Parser;
use domain::peer::PeerAddress;
use infrastructure::event_bus::BroadcastEventBus;
use infrastructure::network::tcp::unidirectional::receiver::TcpInboundListener;
use infrastructure::network::tcp::unidirectional::sender::TcpOutboundConnectionService;
use infrastructure::persistence::file_chat_repository::FileChatRepository;
use infrastructure::persistence::file_peer_repository::FilePeerRepository;
use presentation::app::TuiAppState;
use presentation::user_command::UserCommand;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Parser)]
#[command(name = "peer-pressure", about = "P2P chat application")]
struct Cli {
    /// Port to listen on and connect to peers
    #[arg(long, default_value = "9000")]
    port: u16,

    /// IP address to bind the listener to
    #[arg(long, default_value = "0.0.0.0")]
    bind: String,

    /// Directory to store persistent data
    #[arg(long, default_value = "./data")]
    data_dir: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // 1. Event bus
    let event_bus = Arc::new(BroadcastEventBus::new(256));

    // 2. Repositories
    let peer_repo = Arc::new(FilePeerRepository::new(&cli.data_dir));
    let chat_repo = Arc::new(FileChatRepository::new(&cli.data_dir));
    peer_repo.load().await?;
    chat_repo.load().await?;

    // 3. Use cases
    let receive_message = Arc::new(ReceiveMessage::new(
        chat_repo.clone() as Arc<FileChatRepository>,
        event_bus.clone() as Arc<BroadcastEventBus>,
    ));

    let sender_service = Arc::new(TcpOutboundConnectionService::new(
        cli.port,
        event_bus.clone() as Arc<dyn EventSender>,
    ));

    let send_message_uc = SendMessage::new(
        chat_repo.clone() as Arc<FileChatRepository>,
        sender_service.clone() as Arc<TcpOutboundConnectionService>,
        event_bus.clone() as Arc<BroadcastEventBus>,
    );

    let add_peer_uc = AddPeer::new(
        peer_repo.clone() as Arc<FilePeerRepository>,
        event_bus.clone() as Arc<BroadcastEventBus>,
    );

    let connect_and_resend_uc = ConnectAndResend::new(
        chat_repo.clone() as Arc<FileChatRepository>,
        sender_service.clone() as Arc<TcpOutboundConnectionService>,
        event_bus.clone() as Arc<BroadcastEventBus>,
    );

    // 4. Inbound listener
    let bind_ip: std::net::IpAddr = cli.bind.parse()?;
    let inbound_listener = TcpInboundListener::new(
        bind_ip,
        cli.port,
        receive_message as Arc<dyn InboundMessageHandler>,
        event_bus.clone() as Arc<dyn EventSender>,
    );

    // 5. Command channel (TUI -> main)
    let (command_tx, mut command_rx) = mpsc::channel::<UserCommand>(64);

    // 6. Subscribe TUI to events
    let ui_event_rx = event_bus.subscribe();

    // 7. Load initial state into App
    let mut app = TuiAppState::new();
    if let Ok(peers) = peer_repo.list().await {
        for peer in peers {
            app.add_peer(peer.address());
        }
    }
    if let Ok(chats) = chat_repo.list().await {
        for chat in chats {
            for msg in &chat.messages {
                let sent_by_me = matches!(msg.sent_by(), domain::message::SentBy::Me);
                let delivered =
                    matches!(msg.delivery_status(), domain::message::DeliveryStatus::Sent);
                app.add_message(&chat.peer, &msg.content, sent_by_me, delivered);
            }
        }
    }

    // 8. Spawn inbound listener
    tokio::spawn(async move {
        if let Err(e) = inbound_listener.listen().await {
            eprintln!("Listener error: {e}");
        }
    });

    // 9. Spawn TUI
    tokio::spawn(async move {
        if let Err(e) = presentation::tui::run(app, ui_event_rx, command_tx).await {
            eprintln!("TUI error: {e}");
        }
    });

    // 10. Command processing loop
    while let Some(cmd) = command_rx.recv().await {
        match cmd {
            UserCommand::SendMessage { peer, text } => {
                if let Err(e) = send_message_uc.execute(peer, text).await {
                    eprintln!("Send error: {e}");
                }
            }
            UserCommand::AddPeer { address } => {
                let addr = PeerAddress::new(address.into());
                let _ = add_peer_uc.execute(addr.clone()).await;
                if let Err(e) = sender_service.connect(addr).await {
                    eprintln!("Connect error: {e}");
                }
            }
            UserCommand::ConnectToPeer { peer } => {
                if let Err(e) = connect_and_resend_uc.execute(peer).await {
                    eprintln!("Connect error: {e}");
                }
            }
            UserCommand::Quit => break,
        }
    }

    Ok(())
}
