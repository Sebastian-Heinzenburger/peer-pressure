use application::ports::event_receiver::EventReceiverFactory;
use application::ports::event_sender::EventSender;
use application::ports::inbound_message_handler::InboundMessageReceiver;
use application::ports::repository::chat::ChatRepository;
use application::ports::repository::peer::PeerRepository;
use application::use_cases::{AddPeer, ConnectAndResend, ReceiveMessage, SendMessage};
use clap::Parser;
use domain::peer::PeerAddress;
use infrastructure::event_bus::BroadcastEventBus;
use infrastructure::network::tcp::unidirectional::receiver::TcpInboundListener;
use infrastructure::network::tcp::unidirectional::sender::TcpOutboundConnectionService;
use infrastructure::persistence::file_chat_repository::FileChatRepository;
use infrastructure::persistence::file_peer_repository::FilePeerRepository;
use presentation::tui::app_state::TuiAppState;
use presentation::user_command::UserCommand;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;

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

    let event_bus = Arc::new(BroadcastEventBus::new(256));

    // Repos
    let peer_repo = Arc::new(FilePeerRepository::new(&cli.data_dir));
    let chat_repo = Arc::new(FileChatRepository::new(&cli.data_dir));
    peer_repo.load().await?;
    chat_repo.load().await?;

    // Outbound Network Service
    let sender_service = Arc::new(TcpOutboundConnectionService::new(
        cli.port,
        event_bus.clone() as Arc<dyn EventSender>,
    ));

    // Use cases
    let receive_message = Arc::new(ReceiveMessage::new(
        chat_repo.clone() as Arc<FileChatRepository>,
        event_bus.clone() as Arc<BroadcastEventBus>,
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

    // Inbound Network Listener
    let bind_ip: std::net::IpAddr = cli.bind.parse()?;
    let inbound_listener = TcpInboundListener::new(
        bind_ip,
        cli.port,
        receive_message as Arc<dyn InboundMessageReceiver>,
        event_bus.clone() as Arc<dyn EventSender>,
    );

    // Command channel (TUI -> main)
    let (command_tx, mut command_rx) = mpsc::channel::<UserCommand>(64);

    // Load initial state into Tui App
    let mut tui_app = TuiAppState::new();
    load_state_into_tui_app(&mut tui_app, chat_repo, peer_repo).await;

    // Spawn the inbound listener
    tokio::spawn(async move {
        if let Err(e) = inbound_listener.listen().await {
            eprintln!("Listener error: {e}");
        }
    });

    // Spawn TUI
    tokio::spawn(async move {
        let ui_event_rx = event_bus.subscribe();
        if let Err(e) = presentation::tui::run(tui_app, ui_event_rx, command_tx).await {
            eprintln!("TUI error: {e}");
        }
    });

    command_processing_loop(
        &mut command_rx,
        send_message_uc,
        add_peer_uc,
        connect_and_resend_uc,
        sender_service,
    )
    .await;

    Ok(())
}

async fn load_state_into_tui_app(
    tui_app: &mut TuiAppState,
    chat_repo: Arc<FileChatRepository>,
    peer_repo: Arc<FilePeerRepository>,
) {
    if let Ok(peers) = peer_repo.list().await {
        for peer in peers {
            tui_app.add_peer(peer.address());
        }
    }
    if let Ok(chats) = chat_repo.list().await {
        for chat in chats {
            for msg in &chat.messages {
                let sent_by_me = matches!(msg.sent_by(), domain::message::SentBy::Me);
                let delivered =
                    matches!(msg.delivery_status(), domain::message::DeliveryStatus::Sent);
                tui_app.add_message(&chat.peer, &msg.content, sent_by_me, delivered);
            }
        }
    }
}

async fn command_processing_loop(
    command_rx: &mut Receiver<UserCommand>,
    send_message_uc: SendMessage<
        FileChatRepository,
        TcpOutboundConnectionService,
        BroadcastEventBus,
    >,
    add_peer_uc: AddPeer<FilePeerRepository, BroadcastEventBus>,
    connect_and_resend_uc: ConnectAndResend<
        FileChatRepository,
        TcpOutboundConnectionService,
        BroadcastEventBus,
    >,
    _sender_service: Arc<TcpOutboundConnectionService>,
) {
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
                // if let Err(e) = sender_service.connect(addr).await {
                //     eprintln!("Connect error: {e}");
                // }
            }
            UserCommand::ConnectToPeer { peer } => {
                if let Err(e) = connect_and_resend_uc.execute(peer).await {
                    eprintln!("Connect error: {e}");
                }
            }
            UserCommand::Quit => break,
        }
    }
}
