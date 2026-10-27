use actix_web::web::{self, scope};
use actix_web_httpauth::middleware::HttpAuthentication;
use arcadia_storage::redis::RedisPoolInterface;

use crate::handlers::advent_calendars::config as AdventCalendarsConfig;
use crate::handlers::affiliated_artists::config as AffiliatedArtistsConfig;
use crate::handlers::arcadia_settings::config as ArcadiaSettingsConfig;
use crate::handlers::artists::config as ArtistsConfig;
use crate::handlers::auth::config as AuthConfig;
use crate::handlers::collages::config as CollagesConfig;
use crate::handlers::conversations::config as ConversationsConfig;
use crate::handlers::css_sheets::{
    config as CssSheetsConfig, config_public as CssSheetsPublicConfig,
};
use crate::handlers::donations::config as DonationsConfig;
use crate::handlers::edition_groups::config as EditionGroupsConfig;
use crate::handlers::emojis::config as EmojisConfig;
use crate::handlers::external_db::config as ExternalDbConfig;
use crate::handlers::forum::config as ForumConfig;
use crate::handlers::gifts::config as GiftsConfig;
use crate::handlers::health::health_check;
use crate::handlers::home::config as HomeConfig;
use crate::handlers::image_host::config as ImageHostConfig;
use crate::handlers::invitations::config as InvitationsConfig;
use crate::handlers::maintenance_tools::config as MaintenanceToolsConfig;
use crate::handlers::master_groups::config as MasterGroupsConfig;
use crate::handlers::notifications::config as NotificationsConfig;
use crate::handlers::oauth::config as OAuthConfig;
use crate::handlers::related_forum_threads::config as RelatedForumThreadsConfig;
use crate::handlers::search::config as SearchConfig;
use crate::handlers::series::config as SeriesConfig;
use crate::handlers::shop::config as ShopConfig;
use crate::handlers::site_highlights::config as SiteHighlightsConfig;
use crate::handlers::staff_pms::config as StaffPmsConfig;
use crate::handlers::stats::config as StatsConfig;
use crate::handlers::subscriptions::config as SubscriptionsConfig;
use crate::handlers::title_group_bookmarks::config as BookmarksConfig;
use crate::handlers::title_group_tags::config as TitleGroupTagsConfig;
use crate::handlers::title_groups::config as TitleGroupsConfig;
use crate::handlers::torrent_requests::config as TorrentRequestsConfig;
use crate::handlers::torrents::config as TorrentsConfig;
use crate::handlers::unauthorized_access::config as UnauthorizedAccessConfig;
use crate::handlers::user_applications::config as UserApplicationsConfig;
use crate::handlers::user_badges::config as UserBadgesConfig;
use crate::handlers::user_badges::config_categories as UserBadgeCategoriesConfig;
use crate::handlers::user_classes::config as UserClassesConfig;
use crate::handlers::user_edit_change_logs::config as UserEditChangeLogsConfig;
use crate::handlers::users::config as UsersConfig;
use crate::handlers::wiki::config as WikiConfig;
use crate::middlewares::auth_middleware::authenticate_user;
use crate::middlewares::page_size_cap::cap_page_size_middleware;
use crate::middlewares::side_effects::side_effects_middleware;

pub fn init<R: RedisPoolInterface + 'static>(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check));
    // registered before /api so its middlewares (JWT authentication, side effects envelope) don't
    // apply: these OAuth2 protocol endpoints authenticate the client or its opaque access token
    // themselves, and answer the raw JSON the OAuth2 spec defines
    cfg.service(scope("/api/oauth").configure(OAuthConfig::<R>));
    cfg.service(
        web::scope("/api")
            .wrap(actix_web::middleware::from_fn(side_effects_middleware::<R>))
            .wrap(actix_web::middleware::from_fn(
                cap_page_size_middleware::<R>,
            ))
            .wrap(HttpAuthentication::with_fn(authenticate_user::<R>))
            .service(scope("/css").configure(CssSheetsPublicConfig::<R>))
            .service(scope("/auth").configure(AuthConfig::<R>))
            .service(scope("/users").configure(UsersConfig::<R>))
            .service(scope("/user-applications").configure(UserApplicationsConfig::<R>))
            .service(scope("/user-classes").configure(UserClassesConfig::<R>))
            .service(scope("/user-badges").configure(UserBadgesConfig::<R>))
            .service(scope("/user-badge-categories").configure(UserBadgeCategoriesConfig::<R>))
            .service(scope("/title-group-bookmarks").configure(BookmarksConfig::<R>))
            .service(scope("/title-groups").configure(TitleGroupsConfig::<R>))
            .service(scope("/title-group-tags").configure(TitleGroupTagsConfig::<R>))
            .service(scope("/edition-groups").configure(EditionGroupsConfig::<R>))
            .service(scope("/emojis").configure(EmojisConfig::<R>))
            .service(scope("/search").configure(SearchConfig::<R>))
            .service(scope("/torrents").configure(TorrentsConfig::<R>))
            .service(scope("/torrent-requests").configure(TorrentRequestsConfig::<R>))
            .service(scope("/unauthorized-access").configure(UnauthorizedAccessConfig::<R>))
            .service(scope("/user-edit-change-logs").configure(UserEditChangeLogsConfig::<R>))
            .service(scope("/artists").configure(ArtistsConfig::<R>))
            .service(scope("/affiliated-artists").configure(AffiliatedArtistsConfig::<R>))
            .service(scope("/conversations").configure(ConversationsConfig::<R>))
            .service(scope("/subscriptions").configure(SubscriptionsConfig::<R>))
            .service(scope("/notifications").configure(NotificationsConfig::<R>))
            .service(scope("/series").configure(SeriesConfig::<R>))
            .service(scope("/external-sources").configure(ExternalDbConfig::<R>))
            .service(scope("/forum").configure(ForumConfig::<R>))
            .service(scope("/wiki").configure(WikiConfig::<R>))
            .service(scope("/staff-pms").configure(StaffPmsConfig::<R>))
            .service(scope("/invitations").configure(InvitationsConfig::<R>))
            .service(scope("/home").configure(HomeConfig::<R>))
            .service(scope("/image-host").configure(ImageHostConfig::<R>))
            .service(scope("/master-groups").configure(MasterGroupsConfig::<R>))
            .service(scope("/gifts").configure(GiftsConfig::<R>))
            .service(scope("/shop").configure(ShopConfig::<R>))
            .service(scope("/collages").configure(CollagesConfig::<R>))
            .service(scope("/css-sheets").configure(CssSheetsConfig::<R>))
            .service(scope("/donations").configure(DonationsConfig::<R>))
            .service(scope("/arcadia-settings").configure(ArcadiaSettingsConfig::<R>))
            .service(scope("/site-highlights").configure(SiteHighlightsConfig::<R>))
            .service(scope("/advent-calendars").configure(AdventCalendarsConfig::<R>))
            .service(scope("/related-forum-threads").configure(RelatedForumThreadsConfig::<R>))
            .service(scope("/stats").configure(StatsConfig::<R>))
            .service(scope("/maintenance-tools").configure(MaintenanceToolsConfig::<R>)),
    );
}
