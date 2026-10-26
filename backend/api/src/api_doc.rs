use crate::handlers::artists::delete_artist::DeleteArtistQuery;
use crate::handlers::edition_groups::delete_edition_group::DeleteEditionGroupQuery;
use crate::handlers::title_groups::delete_title_group::DeleteTitleGroupQuery;
use crate::handlers::title_groups::merge_title_groups::MergeTitleGroupsQuery;
use crate::handlers::users::download_user_torrents::UserTorrentsArchiveKind;
use crate::middlewares::api_key_scopes::{api_key_access_of_endpoint, APIKeyAccess};
use actix_web::http::Method;
use arcadia_storage::models::artist::SearchArtistsQuery;
use arcadia_storage::models::bonus_points_log::{
    BonusPointsLog, BonusPointsLogAction, BonusPointsLogOrderByColumn, SearchBonusPointsLogsQuery,
};
use arcadia_storage::models::collage::{
    DeleteCollageEntriesQuery, DeleteCollageQuery, SearchCollagesLiteQuery,
};
use arcadia_storage::models::donation::{
    DeletedDonation, DonationOrderBy, EditedDonation, SearchDonationsQuery, UserCreatedDonation,
};
use arcadia_storage::models::emoji::{
    Emoji, EmojiEnabledUpdate, EmojiUsage, ReorderEmojiEntry, ReorderEmojis,
};
use arcadia_storage::models::forum::{
    DeleteForumCategoryQuery, DeleteForumPostQuery, DeleteForumSubCategoryQuery,
    DeleteForumThreadQuery, ForumPostSearchQuery, ForumSearchQuery, ForumSubCategoryAllowedPoster,
    GetForumSubCategoryAllowedPostersQuery, ReorderForumCategories, ReorderForumCategoryEntry,
    ReorderForumSubCategories, ReorderForumSubCategoryEntry,
};
use arcadia_storage::models::invitation::{
    InvitationSearchOrderByColumn, SearchSentInvitationsQuery,
};
use arcadia_storage::models::reaction::{ContentReaction, ForumPostReactionUsers};
use arcadia_storage::models::series::{SearchSeriesQuery, SeriesSearchOrderByColumn};
use arcadia_storage::models::title_group_comment::{
    TitleGroupCommentSearchQuery, UserTitleGroupCommentSearchQuery,
};
use arcadia_storage::models::title_group_tag::{
    SearchTitleGroupAppliedTagsQuery, SearchTitleGroupTagsQuery,
};
use arcadia_storage::models::torrent::{TorrentDeletionReason, TorrentSearch};
use arcadia_storage::models::torrent_activity::{
    TorrentActivitiesOverview, TorrentActivity, TorrentActivityAndTitleGroup,
    TorrentActivityAndUser, TorrentActivityOfTorrentOrderByColumn, TorrentActivityOrderByColumn,
};
use arcadia_storage::models::torrent_report::DeleteTorrentReportQuery;
use arcadia_storage::models::torrent_request_comment::UserTorrentRequestCommentSearchQuery;
use arcadia_storage::models::user::{
    APIKey, APIKeyScope, CreatedAPIKey, SearchUsersQuery, UserCreatedAPIKey,
    UserPermissionMatchMode, UserSearchOrderBy,
};
use arcadia_storage::models::wiki::{SimilarWikiArticlesLink, WikiArticleResponse};
use arcadia_storage::models::{collage::SearchCollagesQuery, forum::GetForumThreadPostsQuery};
use utoipa::{
    openapi::{
        schema::{Array, Object, Ref, Schema},
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
        ContentBuilder, RefOr, ResponseBuilder,
    },
    Modify, OpenApi, PartialSchema,
};

use arcadia_storage::models::arcadia_settings::AvailableShopItem;
use arcadia_storage::models::shop::{
    BuyFreeleechTokensRequest, BuyUploadRequest, FreeleechTokenDiscountTier,
    FreeleechTokensPriceCalculation, PromotionPricing, ShopPricing, UploadDiscountTier,
    UploadPriceCalculation,
};
use arcadia_storage::models::unauthorized_access::SearchUnauthorizedAccessQuery;
use arcadia_storage::models::user_edit_change_log::{
    DeleteUserEditChangeLogQuery, SearchUserEditChangeLogsQuery,
};

use crate::handlers::auth::irc_auth::{IrcAuthRequest, IrcAuthResponse};
use crate::handlers::image_host::upload_image::{UploadImageForm, UploadImageResponse};
use crate::handlers::notifications::mark_torrent_deletions_as_read::MarkTorrentDeletionsAsReadForm;
use crate::handlers::search::search_title_group_tags_lite::SearchTitleGroupTagsLiteQuery;
use crate::handlers::torrents::move_torrent_to_edition_group::MoveTorrentToEditionGroup;
use crate::handlers::user_applications::get_user_applications::GetUserApplicationsQuery;
use crate::handlers::users::create_irc_account::IrcAccountResponse;
use crate::handlers::users::get_user_torrent_activities_overview::SeedersPerTorrent;
use crate::middlewares::side_effects::SideEffect;
use arcadia_storage::models::conversation::{
    ConversationSearchOrderByColumn, ConversationSearchQuery, ConversationSearchResult,
    MassMessageRequest, MassMessageResult,
};
use arcadia_storage::models::forum_stats::{
    ForumStatsDataPoint, ForumStatsGroupBy, ForumStatsMetric, ForumStatsResponse,
};
use arcadia_storage::models::subscription::SearchSubscriptionsQuery;
use arcadia_storage::models::title_group_stats::{
    TitleGroupAttributeCountDataPoint, TitleGroupStatsResponse,
};
use arcadia_storage::models::torrent_request::{
    SearchTorrentRequestsQuery, TorrentRequestSearchOrderBy,
};
use arcadia_storage::models::torrent_stats::{
    StatsInterval, TitleGroupsPerReleaseYearDataPoint, TorrentDeletionsStatsDataPoint,
    TorrentStatsDataPoint, TorrentStatsGroupBy, TorrentStatsResponse,
};
use arcadia_storage::models::user_application::UserApplicationHierarchy;
use arcadia_storage::models::user_stats::{UserStatsDataPoint, UserStatsResponse};

#[derive(OpenApi)]
#[openapi(
    info(title = "arcadia-backend API"),
    modifiers(&SecurityAddon, &SideEffectsResponseModifier, &APIKeyScopeModifier),
    paths(
        crate::handlers::auth::register::exec,
        crate::handlers::auth::login::exec,
        crate::handlers::auth::logout::exec,
        crate::handlers::auth::refresh_token::exec,
        crate::handlers::auth::oauth_authorize::exec,
        crate::handlers::users::get_user::exec,
        crate::handlers::users::edit_user::exec,
        crate::handlers::users::warn_user::exec,
        crate::handlers::users::remove_user_warnings::exec,
        crate::handlers::search::search_conversations::exec,
        crate::handlers::users::get_me::exec,
        crate::handlers::users::download_user_torrents::exec,
        crate::handlers::users::get_user_settings::exec,
        crate::handlers::users::update_user_settings::exec,
        crate::handlers::users::update_uploaded_torrents_anonymity::exec,
        crate::handlers::users::change_user_class::exec,
        crate::handlers::users::change_user_password::exec,
        crate::handlers::users::create_password_reset_token::exec,
        crate::handlers::auth::reset_password::exec,
        crate::handlers::users::edit_user_permissions::exec,
        crate::handlers::users::get_user_permissions::exec,
        crate::handlers::users::lock_user_class::exec,
        crate::handlers::users::set_user_custom_title::exec,
        crate::handlers::users::get_user_staff_notes::exec,
        crate::handlers::users::create_user_staff_note::exec,
        crate::handlers::users::edit_user_staff_note::exec,
        crate::handlers::users::delete_user_staff_note::exec,
        crate::handlers::users::get_user_registration_ip::exec,
        crate::handlers::users::check_user_registration_ip::exec,
        crate::handlers::users::set_user_registration_ip_check::exec,
        crate::handlers::maintenance_tools::check_missing_registration_ips::exec,
        crate::handlers::users::create_irc_account::exec,
        crate::handlers::users::reset_irc_password::exec,
        crate::handlers::users::get_user_torrent_activities::exec,
        crate::handlers::users::get_user_torrent_activities_overview::exec,
        crate::handlers::users::search_bonus_points_logs::exec,
        crate::handlers::users::create_api_key::exec,
        crate::handlers::users::get_api_keys::exec,
        crate::handlers::users::delete_api_key::exec,
        crate::handlers::shop::buy_promotion::exec,
        crate::handlers::shop::buy_upload::exec,
        crate::handlers::shop::buy_freeleech_tokens::exec,
        crate::handlers::shop::get_pricing::exec,
        crate::handlers::auth::irc_auth::exec,
        crate::handlers::auth::create_user_application::exec,
        crate::handlers::user_applications::get_user_applications::exec,
        crate::handlers::user_applications::update_user_application_status::exec,
        crate::handlers::unauthorized_access::search::exec,
        crate::handlers::user_edit_change_logs::search::exec,
        crate::handlers::user_edit_change_logs::delete_user_edit_change_log::exec,
        crate::handlers::user_edit_change_logs::delete_all_user_edit_change_logs::exec,
        crate::handlers::home::get_home::exec,
        crate::handlers::artists::get_artist::exec,
        crate::handlers::artists::create_artists::exec,
        crate::handlers::artists::edit_artist::exec,
        crate::handlers::artists::delete_artist::exec,
        crate::handlers::affiliated_artists::create_affiliated_artists::exec,
        crate::handlers::affiliated_artists::remove_affiliated_artists::exec,
        crate::handlers::torrents::download_dottorrent_file::exec,
        crate::handlers::torrents::create_torrent::exec,
        crate::handlers::torrents::edit_torrent::exec,
        crate::handlers::torrents::get_upload_information::exec,
        crate::handlers::torrents::get_top_torrents::exec,
        crate::handlers::torrents::delete_torrent::exec,
        crate::handlers::torrents::request_reseed::exec,
        crate::handlers::torrents::create_torrent_report::exec,
        crate::handlers::torrents::delete_torrent_report::exec,
        crate::handlers::torrents::set_torrent_staff_checked::exec,
        crate::handlers::torrents::get_torrent_peers::exec,
        crate::handlers::torrents::get_torrent_activities::exec,
        crate::handlers::torrents::get_torrent_title_group::exec,
        crate::handlers::torrents::edit_torrent_up_down_factors::exec,
        crate::handlers::torrents::move_torrent_to_edition_group::exec,
        crate::handlers::edition_groups::create_edition_group::exec,
        crate::handlers::edition_groups::edit_edition_group::exec,
        crate::handlers::edition_groups::delete_edition_group::exec,
        crate::handlers::invitations::create_invitation::exec,
        crate::handlers::invitations::search_sent_invitations::exec,
        crate::handlers::master_groups::create_master_group::exec,
        crate::handlers::series::create_series::exec,
        crate::handlers::series::get_series::exec,
        crate::handlers::series::edit_series::exec,
        crate::handlers::series::delete_series::exec,
        crate::handlers::series::add_title_group::exec,
        crate::handlers::series::remove_title_group::exec,
        crate::handlers::subscriptions::get_subscription_artist_title_groups::exec,
        crate::handlers::subscriptions::create_subscription_artist_title_groups::exec,
        crate::handlers::subscriptions::remove_subscription_artist_title_groups::exec,
        crate::handlers::subscriptions::get_subscription_collages::exec,
        crate::handlers::subscriptions::create_subscription_collages::exec,
        crate::handlers::subscriptions::remove_subscription_collages::exec,
        crate::handlers::subscriptions::get_subscription_forum_sub_category_threads::exec,
        crate::handlers::subscriptions::create_subscription_forum_sub_category_threads::exec,
        crate::handlers::subscriptions::remove_subscription_forum_sub_category_threads::exec,
        crate::handlers::subscriptions::get_subscription_forum_thread_posts::exec,
        crate::handlers::subscriptions::get_subscription_title_group_torrents::exec,
        crate::handlers::subscriptions::get_subscription_title_group_comments::exec,
        crate::handlers::subscriptions::get_subscription_torrent_request_comments::exec,
        crate::handlers::subscriptions::create_subscription_forum_thread_posts::exec,
        crate::handlers::subscriptions::remove_subscription_forum_thread_posts::exec,
        crate::handlers::subscriptions::create_subscription_title_group_torrents::exec,
        crate::handlers::subscriptions::remove_subscription_title_group_torrents::exec,
        crate::handlers::subscriptions::create_subscription_title_group_comments::exec,
        crate::handlers::subscriptions::remove_subscription_title_group_comments::exec,
        crate::handlers::subscriptions::create_subscription_torrent_request_comments::exec,
        crate::handlers::subscriptions::remove_subscription_torrent_request_comments::exec,
        crate::handlers::notifications::get_notifications::exec,
        crate::handlers::notifications::get_notification_counts::exec,
        crate::handlers::notifications::mark_torrent_deletions_as_read::exec,
        crate::handlers::notifications::mark_reseed_requests_as_read::exec,
        crate::handlers::title_groups::create_title_group_comment::exec,
        crate::handlers::title_groups::edit_title_group_comment::exec,
        crate::handlers::title_groups::delete_title_group_comment::exec,
        crate::handlers::title_groups::create_title_group::exec,
        crate::handlers::title_groups::edit_title_group::exec,
        crate::handlers::title_groups::get_title_group::exec,
        crate::handlers::title_groups::get_title_group_info_lite::exec,
        crate::handlers::title_groups::delete_title_group::exec,
        crate::handlers::title_groups::merge_title_groups::exec,
        crate::handlers::title_groups::link_similar_title_groups::exec,
        crate::handlers::title_groups::unlink_similar_title_groups::exec,
        crate::handlers::title_group_tags::create_tag::exec,
        crate::handlers::title_group_tags::apply_tag::exec,
        crate::handlers::title_group_tags::remove_tag::exec,
        crate::handlers::title_group_tags::delete_tag::exec,
        crate::handlers::title_group_tags::restore_tag::exec,
        crate::handlers::title_group_tags::merge_tags::exec,
        crate::handlers::title_group_tags::edit_tag::exec,
        crate::handlers::title_group_tags::search_applied_tags::exec,
        crate::handlers::search::search_title_group_tags::exec,
        crate::handlers::search::search_title_group_tags_lite::exec,
        crate::handlers::search::search_torrents::exec,
        crate::handlers::search::search_title_group_info_lite::exec,
        crate::handlers::search::search_torrent_requests::exec,
        crate::handlers::search::search_artists::exec,
        crate::handlers::search::search_artists_lite::exec,
        crate::handlers::search::search_collages::exec,
        crate::handlers::search::search_collages_lite::exec,
        crate::handlers::search::search_series::exec,
        crate::handlers::search::search_series_lite::exec,
        crate::handlers::search::search_forum::exec,
        crate::handlers::search::search_forum_posts::exec,
        crate::handlers::search::search_title_group_comments::exec,
        crate::handlers::search::search_user_title_group_comments::exec,
        crate::handlers::search::search_user_torrent_request_comments::exec,
        crate::handlers::search::search_users::exec,
        crate::handlers::search::search_users_lite::exec,
        crate::handlers::torrent_requests::create_torrent_request::exec,
        crate::handlers::torrent_requests::get_torrent_request::exec,
        crate::handlers::torrent_requests::fill_torrent_request::exec,
        crate::handlers::torrent_requests::create_torrent_request_vote::exec,
        crate::handlers::torrent_requests::create_torrent_request_comment::exec,
        crate::handlers::torrent_requests::delete_torrent_request_comment::exec,
        crate::handlers::torrent_requests::delete_torrent_request::exec,
        crate::handlers::gifts::create_gift::exec,
        crate::handlers::donations::search_donations::exec,
        crate::handlers::donations::create_donation::exec,
        crate::handlers::donations::edit_donation::exec,
        crate::handlers::donations::delete_donation::exec,
        crate::handlers::forum::get_forum::exec,
        crate::handlers::forum::create_forum_category::exec,
        crate::handlers::forum::edit_forum_category::exec,
        crate::handlers::forum::edit_forum_sub_category::exec,
        crate::handlers::forum::create_forum_sub_category::exec,
        crate::handlers::forum::get_forum_sub_category_threads::exec,
        crate::handlers::forum::get_forum_thread::exec,
        crate::handlers::forum::get_forum_thread_posts::exec,
        crate::handlers::forum::create_forum_thread::exec,
        crate::handlers::forum::edit_forum_thread::exec,
        crate::handlers::forum::pin_forum_thread::exec,
        crate::handlers::forum::create_forum_post::exec,
        crate::handlers::forum::create_forum_post_reaction::exec,
        crate::handlers::forum::delete_forum_post_reaction::exec,
        crate::handlers::forum::get_forum_post_reaction_users::exec,
        crate::handlers::forum::create_forum_poll::exec,
        crate::handlers::forum::create_forum_poll_vote::exec,
        crate::handlers::forum::edit_forum_post::exec,
        crate::handlers::forum::delete_forum_category::exec,
        crate::handlers::forum::delete_forum_sub_category::exec,
        crate::handlers::forum::delete_forum_thread::exec,
        crate::handlers::forum::delete_forum_post::exec,
        crate::handlers::forum::get_forum_sub_category_allowed_posters::exec,
        crate::handlers::forum::add_forum_sub_category_allowed_poster::exec,
        crate::handlers::forum::remove_forum_sub_category_allowed_poster::exec,
        crate::handlers::forum::reorder_forum_category::exec,
        crate::handlers::forum::reorder_forum_sub_category::exec,
        crate::handlers::wiki::create_wiki_article::exec,
        crate::handlers::wiki::get_wiki_article::exec,
        crate::handlers::wiki::edit_wiki_article::exec,
        crate::handlers::wiki::link_similar_wiki_articles::exec,
        crate::handlers::wiki::unlink_similar_wiki_articles::exec,
        crate::handlers::search::search_wiki::exec,
        crate::handlers::conversations::create_conversation::exec,
        crate::handlers::conversations::create_mass_conversation::exec,
        crate::handlers::conversations::get_conversation::exec,
        crate::handlers::conversations::create_conversation_message::exec,
        crate::handlers::staff_pms::create_staff_pm::exec,
        crate::handlers::staff_pms::create_staff_pm_message::exec,
        crate::handlers::staff_pms::get_staff_pm::exec,
        crate::handlers::staff_pms::list_staff_pms::exec,
        crate::handlers::staff_pms::resolve_staff_pm::exec,
        crate::handlers::staff_pms::unresolve_staff_pm::exec,
        crate::handlers::collages::create_collage::exec,
        crate::handlers::collages::create_collage_entries::exec,
        crate::handlers::collages::get_collage::exec,
        crate::handlers::collages::edit_collage::exec,
        crate::handlers::collages::delete_collage::exec,
        crate::handlers::collages::delete_collage_entries::exec,
        crate::handlers::css_sheets::create_css_sheet::exec,
        crate::handlers::css_sheets::edit_css_sheet::exec,
        crate::handlers::css_sheets::get_css_sheet_content::exec,
        crate::handlers::css_sheets::get_css_sheets::exec,
        crate::handlers::css_sheets::get_css_sheet::exec,
        crate::handlers::arcadia_settings::get_arcadia_settings::exec,
        crate::handlers::arcadia_settings::get_public_arcadia_settings::exec,
        crate::handlers::arcadia_settings::update_arcadia_settings::exec,
        crate::handlers::site_highlights::list_site_highlights::exec,
        crate::handlers::site_highlights::create_site_highlight::exec,
        crate::handlers::site_highlights::edit_site_highlight::exec,
        crate::handlers::site_highlights::delete_site_highlight::exec,
        crate::handlers::related_forum_threads::create_related_forum_thread::exec,
        crate::handlers::related_forum_threads::delete_related_forum_thread::exec,
        crate::handlers::external_db::get_external_source_data::exec,
        crate::handlers::title_group_bookmarks::create_title_group_bookmark::exec,
        crate::handlers::title_group_bookmarks::edit_title_group_bookmark::exec,
        crate::handlers::title_group_bookmarks::get_title_group_bookmark::exec,
        crate::handlers::title_group_bookmarks::remove_title_group_bookmark::exec,
        crate::handlers::user_classes::create_user_class::exec,
        crate::handlers::user_classes::edit_user_class::exec,
        crate::handlers::user_classes::delete_user_class::exec,
        crate::handlers::user_classes::get_user_classes::exec,
        crate::handlers::stats::get_torrent_stats::exec,
        crate::handlers::stats::get_forum_stats::exec,
        crate::handlers::stats::get_user_stats::exec,
        crate::handlers::image_host::upload_image::exec,
        crate::handlers::user_badges::create_user_badge::exec,
        crate::handlers::user_badges::edit_user_badge::exec,
        crate::handlers::user_badges::delete_user_badge::exec,
        crate::handlers::user_badges::list_user_badges::exec,
        crate::handlers::user_badges::award_user_badge::exec,
        crate::handlers::user_badges::revoke_user_earned_badge::exec,
        crate::handlers::user_badges::create_user_badge_category::exec,
        crate::handlers::user_badges::edit_user_badge_category::exec,
        crate::handlers::user_badges::delete_user_badge_category::exec,
        crate::handlers::user_badges::list_user_badge_categories::exec,
        crate::handlers::maintenance_tools::rehash_torrents::exec,
        crate::handlers::maintenance_tools::recompute_cached_amounts::exec,
        crate::handlers::emojis::get_emojis::exec,
        crate::handlers::emojis::get_emojis_usage::exec,
        crate::handlers::emojis::get_emoji_image::exec,
        crate::handlers::emojis::create_emoji::exec,
        crate::handlers::emojis::edit_emoji::exec,
        crate::handlers::emojis::delete_emoji::exec,
        crate::handlers::emojis::reorder_emojis::exec,
        crate::handlers::emojis::set_emoji_enabled::exec,
    ),
    components(schemas(
        UserTorrentsArchiveKind,
        APIKey,
        APIKeyScope,
        CreatedAPIKey,
        UserCreatedAPIKey,
        GetUserApplicationsQuery,
        UserApplicationHierarchy,
        SearchUnauthorizedAccessQuery,
        SearchUserEditChangeLogsQuery,
        DeleteUserEditChangeLogQuery,
        SearchTorrentRequestsQuery,
        TorrentRequestSearchOrderBy,
        SearchArtistsQuery,
        DeleteArtistQuery,
        DeleteEditionGroupQuery,
        DeleteTitleGroupQuery,
        MergeTitleGroupsQuery,
        SearchCollagesQuery,
        SearchCollagesLiteQuery,
        DeleteCollageQuery,
        DeleteCollageEntriesQuery,
        SearchSeriesQuery,
        GetForumThreadPostsQuery,
        TorrentSearch,
        ForumSearchQuery,
        ForumPostSearchQuery,
        TitleGroupCommentSearchQuery,
        UserTitleGroupCommentSearchQuery,
        UserTorrentRequestCommentSearchQuery,
        DeleteForumCategoryQuery,
        DeleteForumSubCategoryQuery,
        DeleteForumThreadQuery,
        DeleteForumPostQuery,
        ForumSubCategoryAllowedPoster,
        GetForumSubCategoryAllowedPostersQuery,
        ReorderForumCategories,
        ReorderForumCategoryEntry,
        ReorderForumSubCategories,
        ReorderForumSubCategoryEntry,
        SearchTitleGroupTagsLiteQuery,
        SearchTitleGroupTagsQuery,
        SearchTitleGroupAppliedTagsQuery,
        SearchDonationsQuery,
        UserCreatedDonation,
        EditedDonation,
        DeletedDonation,
        DonationOrderBy,
        SearchUsersQuery,
        UserSearchOrderBy,
        UserPermissionMatchMode,
        SearchSentInvitationsQuery,
        InvitationSearchOrderByColumn,
        SearchSeriesQuery,
        SeriesSearchOrderByColumn,
        DeleteTorrentReportQuery,
        TorrentActivity,
        TorrentActivityAndTitleGroup,
        TorrentActivityAndUser,
        TorrentActivityOrderByColumn,
        TorrentActivityOfTorrentOrderByColumn,
        BonusPointsLog,
        BonusPointsLogAction,
        BonusPointsLogOrderByColumn,
        SearchBonusPointsLogsQuery,
        TorrentActivitiesOverview,
        SeedersPerTorrent,
        ShopPricing,
        AvailableShopItem,
        PromotionPricing,
        BuyUploadRequest,
        BuyFreeleechTokensRequest,
        UploadDiscountTier,
        FreeleechTokenDiscountTier,
        UploadPriceCalculation,
        FreeleechTokensPriceCalculation,
        TorrentStatsResponse,
        TorrentStatsDataPoint,
        TorrentDeletionsStatsDataPoint,
        TitleGroupsPerReleaseYearDataPoint,
        StatsInterval,
        TorrentStatsGroupBy,
        ForumStatsResponse,
        ForumStatsDataPoint,
        ForumStatsGroupBy,
        ForumStatsMetric,
        UserStatsResponse,
        UserStatsDataPoint,
        TitleGroupStatsResponse,
        TitleGroupAttributeCountDataPoint,
        SearchSubscriptionsQuery,
        UploadImageForm,
        UploadImageResponse,
        IrcAccountResponse,
        IrcAuthRequest,
        IrcAuthResponse,
        MoveTorrentToEditionGroup,
        TorrentDeletionReason,
        MarkTorrentDeletionsAsReadForm,
        SimilarWikiArticlesLink,
        WikiArticleResponse,
        arcadia_storage::models::title_group::SimilarTitleGroupsLink,
        arcadia_storage::models::title_group::SimilarTitleGroupLite,
        ConversationSearchQuery,
        ConversationSearchResult,
        ConversationSearchOrderByColumn,
        MassMessageRequest,
        MassMessageResult,
        arcadia_storage::models::user_badge::UserBadge,
        arcadia_storage::models::user_badge::UserBadgeCategory,
        arcadia_storage::models::user_badge::UserBadgeType,
        arcadia_storage::models::user_badge::UserBadgeListItem,
        arcadia_storage::models::user_badge::UserCreatedUserBadge,
        arcadia_storage::models::user_badge::EditedUserBadge,
        arcadia_storage::models::user_badge::UserCreatedUserBadgeCategory,
        arcadia_storage::models::user_badge::EditedUserBadgeCategory,
        arcadia_storage::models::user_badge::UserBadgeManualAward,
        arcadia_storage::models::user_badge::UserEarnedBadge,
        arcadia_storage::models::user_badge::UserEarnedBadgeWithDetails,
        arcadia_storage::models::user_badge::UserBadgeCriteria,
        arcadia_storage::models::site_highlight::SiteHighlight,
        arcadia_storage::models::site_highlight::SiteHighlightForHome,
        arcadia_storage::models::site_highlight::SiteHighlightItemType,
        arcadia_storage::models::site_highlight::CreateSiteHighlight,
        arcadia_storage::models::site_highlight::EditSiteHighlight,
        arcadia_storage::models::forum::ForumThreadEnrichedHierarchy,
        arcadia_storage::models::forum::UserCreatedForumPoll,
        arcadia_storage::models::forum::UserCreatedForumPollVote,
        arcadia_storage::models::forum::ForumPoll,
        arcadia_storage::models::forum::ForumPollHierarchy,
        arcadia_storage::models::forum::ForumPollOptionResult,
        arcadia_storage::models::forum::RelatedForumThread,
        arcadia_storage::models::forum::CreateRelatedForumThread,
        arcadia_storage::models::forum::DeleteRelatedForumThreadQuery,
        arcadia_storage::models::series::SeriesEnriched,
        arcadia_storage::models::artist::ArtistEnriched,
        crate::handlers::maintenance_tools::rehash_torrents::RehashTorrentsResponse,
        crate::handlers::maintenance_tools::check_missing_registration_ips::CheckMissingRegistrationIpsResponse,
        Emoji,
        EmojiUsage,
        ReorderEmojis,
        ReorderEmojiEntry,
        EmojiEnabledUpdate,
        ContentReaction,
        ForumPostReactionUsers,
        crate::handlers::auth::oauth_authorize::OAuthAuthorize,
        crate::handlers::auth::oauth_authorize::OAuthAuthorizeResponse,
        crate::handlers::oauth::OAuthErrorResponse,
        // exported here for the scraper plugins to use
        crate::handlers::scrapers::ScrapedExternalData,
        crate::handlers::scrapers::ScrapedAffiliatedArtist,
    ),)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // we can unwrap safely since there already is components registered.
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "http",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        )
    }
}

/// Documents, on every endpoint, the API key scope it can be reached with. The scopes
/// themselves are only declared in [`crate::middlewares::api_key_scopes`].
struct APIKeyScopeModifier;

impl Modify for APIKeyScopeModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        for (path, path_item) in openapi.paths.paths.iter_mut() {
            let operations = [
                (Method::GET, &mut path_item.get),
                (Method::PUT, &mut path_item.put),
                (Method::POST, &mut path_item.post),
                (Method::DELETE, &mut path_item.delete),
                (Method::PATCH, &mut path_item.patch),
            ]
            .into_iter()
            .filter_map(|(method, operation)| operation.as_mut().map(|op| (method, op)));

            for (method, operation) in operations {
                let description = match api_key_access_of_endpoint(&method, path) {
                    APIKeyAccess::Scope(scope) => {
                        format!(
                            "Reachable with an API key of the `{}` scope.",
                            scope.as_ref()
                        )
                    }
                    APIKeyAccess::EveryScope => {
                        "Reachable with an API key of any scope.".to_string()
                    }
                    APIKeyAccess::NoAuthenticationRequired => {
                        "Reachable without authenticating.".to_string()
                    }
                    APIKeyAccess::Forbidden | APIKeyAccess::Unmapped => {
                        "Not reachable with an API key.".to_string()
                    }
                };

                operation.description = Some(match operation.description.take() {
                    Some(existing) => format!("{existing}\n\n{description}"),
                    None => description,
                });
            }
        }
    }
}

struct SideEffectsResponseModifier;

impl Modify for SideEffectsResponseModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components
            .schemas
            .insert("SideEffect".to_string(), SideEffect::schema());

        for path_item in openapi.paths.paths.values_mut() {
            let operations: Vec<&mut utoipa::openapi::path::Operation> = [
                &mut path_item.get,
                &mut path_item.put,
                &mut path_item.post,
                &mut path_item.delete,
                &mut path_item.options,
                &mut path_item.head,
                &mut path_item.patch,
                &mut path_item.trace,
            ]
            .into_iter()
            .filter_map(|op| op.as_mut())
            .collect();

            for operation in operations {
                for (status_code, response_ref) in operation.responses.responses.iter_mut() {
                    let is_success = status_code
                        .parse::<u16>()
                        .map(|code| (200..300).contains(&code))
                        .unwrap_or(false);
                    if !is_success {
                        continue;
                    }

                    let RefOr::T(response) = response_ref else {
                        continue;
                    };

                    let Some(RefOr::T(content)) = response.content.get("application/json") else {
                        continue;
                    };

                    let Some(original_schema) = content.schema.clone() else {
                        continue;
                    };

                    let wrapped_schema = Schema::Object(
                        Object::builder()
                            .property(
                                "side_effects",
                                Schema::Array(
                                    Array::builder()
                                        .items(RefOr::Ref(Ref::from_schema_name("SideEffect")))
                                        .build(),
                                ),
                            )
                            .required("side_effects")
                            .property("data", original_schema)
                            .required("data")
                            .build(),
                    );

                    *response_ref = RefOr::T(
                        ResponseBuilder::new()
                            .description(response.description.clone())
                            .content(
                                "application/json",
                                ContentBuilder::new().schema(Some(wrapped_schema)).build(),
                            )
                            .build(),
                    );
                }
            }
        }
    }
}
