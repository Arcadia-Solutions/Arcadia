use actix_web::{web::Data, App, HttpServer};
use arcadia_tracker::{api_doc::ApiDoc, config::Config, routes::init, scheduler, Tracker};
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Name reported to the telemetry collector for this service.
const OTEL_SERVICE_NAME: &str = "arcadia-tracker";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = arcadia_shared::config::load::<Config>();

    arcadia_shared::telemetry::init_telemetry(
        &config.telemetry,
        &config.tracker.log_level,
        OTEL_SERVICE_NAME,
    );

    let telemetry_enabled = config.telemetry.otlp_endpoint.is_some();
    let server_url = format!("{}:{}", config.tracker.host, config.tracker.port);
    println!("Server running at http://{server_url}");

    let arc = Data::new(Tracker::new(config).await);

    if telemetry_enabled {
        arcadia_tracker::metrics::register(&arc, OTEL_SERVICE_NAME);
    } else {
        log::info!("telemetry.otlp_endpoint is not set, skipping metrics registration");
    }

    // Starts scheduler to automate flushing updates
    // to database and inactive peer removal.
    let scheduler_handle = tokio::spawn({
        let arc = arc.clone();

        async move {
            scheduler::handle(&arc).await;
        }
    });

    let arc2 = arc.clone();
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(arc2.clone())
            .configure(init) // Initialize routes
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/swagger-json/openapi.json", ApiDoc::openapi()),
            )
    })
    .bind(server_url)?
    .run();

    server.await?;

    // Flush all remaining updates before shutting down.
    let max_flushes = 1000;
    let mut flushes = 0;

    // stop the scheduler to avoid race conditions
    scheduler_handle.abort();

    while flushes < max_flushes
        && (!arc.peer_updates.lock().is_empty()
            || !arc.torrent_updates.lock().is_empty()
            || !arc.user_updates.lock().is_empty()
            || !arc.announce_error_updates.lock().is_empty())
    {
        scheduler::flush(&arc).await;
        flushes += 1;
    }

    if flushes == max_flushes {
        log::error!("Graceful shutdown failed");
    } else {
        log::info!("Graceful shutdown succeeded");
    }

    // Blocking flush, kept off the runtime workers that drive the gRPC exporters
    let _ = tokio::task::spawn_blocking(arcadia_shared::telemetry::shutdown_telemetry).await;

    Ok(())
}
