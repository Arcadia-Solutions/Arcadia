use actix_web::web::{self, scope};
use actix_web_httpauth::middleware::HttpAuthentication;
use arcadia_storage::redis::RedisPoolInterface;

use crate::handlers::*;
use crate::middlewares::auth_middleware::authenticate_user;
use crate::middlewares::page_size_cap::cap_page_size_middleware;
use crate::middlewares::side_effects::side_effects_middleware;

pub fn init<R: RedisPoolInterface + 'static>(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health::health_check));
    // registered before /api so its middlewares (JWT authentication, side effects envelope) don't
    // apply: these OAuth2 protocol endpoints authenticate the client or its opaque access token
    // themselves, and answer the raw JSON the OAuth2 spec defines
    cfg.service(scope("/api/oauth").configure(oauth::config::<R>));
    cfg.service(
        web::scope("/api")
            .wrap(actix_web::middleware::from_fn(side_effects_middleware::<R>))
            .wrap(actix_web::middleware::from_fn(
                cap_page_size_middleware::<R>,
            ))
            .wrap(HttpAuthentication::with_fn(authenticate_user::<R>))
            .service(scope("/css").configure(css_sheets::config_public::<R>))
            .service(scope("/auth").configure(auth::config::<R>))
            .service(scope("/users").configure(users::config::<R>))
            .service(scope("/user-applications").configure(user_applications::config::<R>))
            .service(scope("/user-classes").configure(user_classes::config::<R>))
            .service(scope("/user-badges").configure(user_badges::config::<R>))
            .service(scope("/user-badge-categories").configure(user_badges::config_categories::<R>))
            .service(scope("/title-group-bookmarks").configure(title_group_bookmarks::config::<R>))
            .service(scope("/title-groups").configure(title_groups::config::<R>))
            .service(scope("/title-group-tags").configure(title_group_tags::config::<R>))
            .service(scope("/edition-groups").configure(edition_groups::config::<R>))
            .service(scope("/emojis").configure(emojis::config::<R>))
            .service(scope("/search").configure(search::config::<R>))
            .service(scope("/torrents").configure(torrents::config::<R>))
            .service(scope("/torrent-requests").configure(torrent_requests::config::<R>))
            .service(scope("/unauthorized-access").configure(unauthorized_access::config::<R>))
            .service(scope("/user-edit-change-logs").configure(user_edit_change_logs::config::<R>))
            .service(scope("/artists").configure(artists::config::<R>))
            .service(scope("/affiliated-artists").configure(affiliated_artists::config::<R>))
            .service(scope("/conversations").configure(conversations::config::<R>))
            .service(scope("/subscriptions").configure(subscriptions::config::<R>))
            .service(scope("/notifications").configure(notifications::config::<R>))
            .service(scope("/series").configure(series::config::<R>))
            .service(scope("/external-sources").configure(external_db::config::<R>))
            .service(scope("/forum").configure(forum::config::<R>))
            .service(scope("/wiki").configure(wiki::config::<R>))
            .service(scope("/staff-pms").configure(staff_pms::config::<R>))
            .service(scope("/invitations").configure(invitations::config::<R>))
            .service(scope("/home").configure(home::config::<R>))
            .service(scope("/image-host").configure(image_host::config::<R>))
            .service(scope("/master-groups").configure(master_groups::config::<R>))
            .service(scope("/gifts").configure(gifts::config::<R>))
            .service(scope("/shop").configure(shop::config::<R>))
            .service(scope("/collages").configure(collages::config::<R>))
            .service(scope("/css-sheets").configure(css_sheets::config::<R>))
            .service(scope("/donations").configure(donations::config::<R>))
            .service(scope("/arcadia-settings").configure(arcadia_settings::config::<R>))
            .service(scope("/site-highlights").configure(site_highlights::config::<R>))
            .service(scope("/advent-calendars").configure(advent_calendars::config::<R>))
            .service(scope("/related-forum-threads").configure(related_forum_threads::config::<R>))
            .service(scope("/stats").configure(stats::config::<R>))
            .service(scope("/maintenance-tools").configure(maintenance_tools::config::<R>)),
    );
}
