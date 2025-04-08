use crate::config::AppConfig;
use crate::events::{self, ProgressEvent};
use crate::handlers;
use actix_files::Files;
use actix_web::dev::ServiceResponse;
use actix_web::http::{StatusCode, header};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{App, HttpResponse, HttpServer, Result, web};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, oneshot};
use tokio::task::JoinHandle;
use tracing::{debug, info};
use tracing_actix_web::TracingLogger;

pub struct AppState {
    pub shutdown_sender: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    pub event_channel: broadcast::Sender<ProgressEvent>,
}

pub async fn start_server(config: &AppConfig) -> std::io::Result<JoinHandle<std::io::Result<()>>> {
    // Create a channel for shutting down the server
    let (tx, rx) = oneshot::channel::<()>();

    // Wrap the Sender in Arc<Mutex<Option<...>>> so it can be shared and taken
    let shared_tx = Arc::new(Mutex::new(Some(tx)));

    // Create a broadcast channel for SSE events
    let (event_tx, _) = broadcast::channel::<ProgressEvent>(100);

    let app_state = web::Data::new(AppState {
        shutdown_sender: Arc::clone(&shared_tx),
        event_channel: event_tx,
    });

    info!(
        "Server is running on {}:{}",
        config.server_host, config.server_port
    );
    debug!("Debug logging is enabled");

    // Clone the values we need inside the closure to avoid lifetime issues
    let static_files_dir = config.static_files_dir.clone();
    let server_host = config.server_host.clone();
    let server_port = config.server_port;

    // Start the server
    let server = HttpServer::new(move || {
        App::new()
            // Logging for Actix with more details
            .wrap(TracingLogger::default())
            // Add error handlers for 404 Not Found errors to redirect to root
            .wrap(ErrorHandlers::new().handler(StatusCode::NOT_FOUND, redirect_to_root))
            // Store the app state
            .app_data(app_state.clone())
            // Serve the download endpoint and static files
            .route("/download", web::post().to(handlers::download_and_extract))
            .route("/upload", web::post().to(handlers::upload_and_extract))
            .route("/events", web::get().to(events::sse_events))
            .route("/dev-info", web::get().to(handlers::info))
            .service(Files::new("/", &static_files_dir).index_file("index.html"))
    })
    .bind((server_host, server_port))?
    .run();

    let server_handle = server.handle();

    // Spawn a task to wait for the shutdown signal
    tokio::spawn(async move {
        // If we receive the shutdown signal, stop the server gracefully
        if rx.await.is_ok() {
            info!("Received shutdown signal, stopping Actix server");
            server_handle.stop(true).await;
            info!("Actix server stopped, transitioning to process management mode");
        }
    });

    // Setup signal handlers for SIGTERM and SIGINT
    setup_signal_handlers();

    Ok(tokio::spawn(server))
}

// Function to redirect 404 responses to the root path
fn redirect_to_root<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let response = HttpResponse::Found()
        .insert_header((header::LOCATION, "/"))
        .finish()
        .map_into_right_body();

    Ok(ErrorHandlerResponse::Response(res.into_response(response)))
}

fn setup_signal_handlers() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        use tracing::info;

        // Handle SIGTERM
        tokio::spawn(async move {
            let mut sigterm = signal(SignalKind::terminate()).unwrap();
            sigterm.recv().await;
            info!("Received SIGTERM, initiating shutdown");
            std::process::exit(0);
        });

        // Handle SIGINT
        tokio::spawn(async move {
            let mut sigint = signal(SignalKind::interrupt()).unwrap();
            sigint.recv().await;
            info!("Received SIGINT, initiating shutdown");
            std::process::exit(0);
        });
    }
}
