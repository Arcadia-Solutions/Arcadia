use actix_cors::Cors;
use actix_web::{web::Data, App, HttpServer};
use arcadia_api::routes::init;
use arcadia_api::{api_doc::ApiDoc, config::Config, Arcadia};
use arcadia_periodic_tasks::periodic_tasks::scheduler::run_periodic_tasks;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::redis::{RedisPool, RedisPoolInterface};
use std::sync::Arc;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Name reported to the telemetry collector for this service.
const OTEL_SERVICE_NAME: &str = "arcadia-api";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // `arcadia-api openapi` prints the spec without needing a config or database,
    // so CI can check the generated frontend client against it.
    if std::env::args().nth(1).as_deref() == Some("openapi") {
        println!("{}", ApiDoc::openapi().to_pretty_json()?);
        return Ok(());
    }
    let config = arcadia_shared::config::load::<Config>();

    arcadia_shared::telemetry::init_telemetry(
        &config.telemetry,
        &config.api.log_level,
        OTEL_SERVICE_NAME,
    );

    if config.telemetry.otlp_endpoint.is_some() {
        arcadia_common::metrics::register(OTEL_SERVICE_NAME);
    } else {
        log::info!("telemetry.otlp_endpoint is not set, skipping metrics registration");
    }

    let server_url = format!("{}:{}", config.api.host, config.api.port);
    println!("Server running at http://{server_url}");

    if config.api.tmdb_api_key.is_none() {
        println!("api.tmdb_api_key is not set. TMDB data fetching won't be available")
    }

    if config.smtp.is_enabled() {
        println!("Email service configured and enabled");
    } else {
        println!("Email service not configured - emails will be skipped");
    }

    if config.ergo.is_enabled() {
        println!("Ergo IRC integration configured and enabled");
    } else {
        println!("Ergo IRC integration not configured - IRC account provisioning will be skipped");
    }

    let tracker_config = arcadia_storage::connection_pool::TrackerConfig {
        url_internal: config.tracker.url_internal.clone(),
        api_key: config.tracker.api_key.clone(),
    };

    if config.api.http_proxy.is_some() {
        println!(
            "api.http_proxy configured, outgoing requests to external services will be proxied"
        );
    }

    let internal_http_client = arcadia_api::build_no_proxy_http_client();
    let pool = Arc::new(
        ConnectionPool::try_new(&config.database.url(), tracker_config, internal_http_client)
            .await
            .expect("db connection"),
    );

    // Initialize and start periodic tasks before starting the web server
    // This ensures that if periodic tasks fail to initialize (e.g., an invalid bonus formula),
    // the entire application fails to start
    let _scheduler = run_periodic_tasks(Arc::clone(&pool), config.periodic_tasks.clone())
        .await
        .expect("Failed to initialize periodic tasks");

    let redis_pool = Arc::new(RedisPool::new(
        &config.redis.host,
        &config.redis.password,
        config.redis.port,
    ));
    // Opening a connection authenticates, so wrong host or credentials fail here instead of at first request
    redis_pool
        .connection()
        .await
        .expect("failed to connect to redis, check the redis configuration");

    // Load settings from database on startup
    let settings = pool
        .get_arcadia_settings()
        .await
        .expect("failed to load arcadia settings from database");

    let arc = Data::new(Arcadia::new(
        Arc::clone(&pool),
        Arc::clone(&redis_pool),
        config,
        settings,
    ));
    let server = HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(TracingLogger::default())
            .wrap(cors)
            .app_data(arc.clone())
            .configure(init::<RedisPool>) // Initialize routes
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/swagger-json/openapi.json", ApiDoc::openapi()),
            )
    })
    .bind(server_url)?
    .run();

    let result = server.await;
    // Blocking flush, kept off the runtime workers that drive the gRPC exporters
    let _ = tokio::task::spawn_blocking(arcadia_shared::telemetry::shutdown_telemetry).await;
    result
}
