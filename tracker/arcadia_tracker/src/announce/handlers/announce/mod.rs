pub mod handle_announce;

use actix_web::web::{ServiceConfig, get, resource};

/// Registers the primary announce endpoint: `/{passkey}` (mounted under `/announce` scope).
pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(resource("/{passkey}").route(get().to(self::handle_announce::exec)));
}

/// Registers the legacy fallback announce endpoint: `/announce` (mounted under `/{passkey}` scope).
pub fn legacy_config(cfg: &mut ServiceConfig) {
    cfg.service(resource("/announce").route(get().to(self::handle_announce::exec)));
}
