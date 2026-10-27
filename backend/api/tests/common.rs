use actix_http::Request;
use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceResponse},
    http::{
        header::{HeaderValue, TryIntoHeaderPair, AUTHORIZATION, CONTENT_TYPE},
        StatusCode,
    },
    test, web, App, Error,
};
use arcadia_api::{config::Config, Arcadia};
use arcadia_storage::models::{notification::NotificationEvent, user::Login};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::user::{LoginResponse, User},
    redis::RedisPoolInterface,
};
use serde::{de::DeserializeOwned, Deserialize};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Deserialize)]
pub struct Profile {
    pub user: User,
}

pub async fn create_test_app<R: RedisPoolInterface + 'static>(
    pool: Arc<ConnectionPool>,
    redis_pool: R,
) -> impl Service<Request, Response = ServiceResponse, Error = Error> {
    create_test_app_with_notifications(pool, redis_pool).await.0
}

// Also returns a receiver subscribed to the app's notification broadcast before any request is made.
pub async fn create_test_app_with_notifications<R: RedisPoolInterface + 'static>(
    pool: Arc<ConnectionPool>,
    redis_pool: R,
) -> (
    impl Service<Request, Response = ServiceResponse, Error = Error>,
    broadcast::Receiver<NotificationEvent>,
) {
    create_test_app_with_config(pool, redis_pool, arcadia_shared::config::load::<Config>()).await
}

// For tests needing configuration the test config file does not provide.
pub async fn create_test_app_with_config<R: RedisPoolInterface + 'static>(
    pool: Arc<ConnectionPool>,
    redis_pool: R,
    config: Config,
) -> (
    impl Service<Request, Response = ServiceResponse, Error = Error>,
    broadcast::Receiver<NotificationEvent>,
) {
    // Load settings from database for tests
    let settings = pool
        .get_arcadia_settings()
        .await
        .expect("failed to load arcadia settings from database");

    let arc = Arcadia::<R>::new(pool, Arc::new(redis_pool), config, settings);

    let notification_receiver = arc.notification_sender.subscribe();

    // TODO: CORS?
    let service = test::init_service(
        App::new()
            .app_data(web::Data::new(arc))
            .configure(arcadia_api::routes::init::<R>),
    )
    .await;
    (service, notification_receiver)
}

pub enum TestUser {
    // Requires the "with_test_users" fixture.
    Standard,
    EditArtist,
    EditSeries,
    EditTitleGroupComment,
    CreateCssSheet,
    EditCssSheet,
    CreateForumCategory,
    EditForumCategory,
    CreateForumSubCategory,
    EditForumSubCategory,
    EditForumThread,
    EditForumPost,
    DeleteForumCategory,
    DeleteForumSubCategory,
    DeleteForumThread,
    DeleteForumPost,
    ForumCategoryFlow,
    ForumSubCategoryFlow,
    CreateUserClass,
    EditUserClass,
    DeleteUserClass,
    SetAndViewUserPermissions,
    LockUserClass,
    ChangeUserClass,
    EditArcadiaSettings,
    CreateDonation,
    EditDonation,
    DeleteDonation,
    SearchDonation,
    SearchUsers,
    SearchUsersWithPermissions,
    SendMassPmWithPermissions,
    WarnBanUser,
    SearchUnauthorizedAccess,
    DeleteArtist,
    SetTorrentStaffChecked,
    SearchUserEditChangeLogs,
    EditEditionGroup,
    EditCollage,
    DeleteCollage,
    DeleteCollageEntry,
    DeleteTitleGroup,
    ViewTorrentPeersAndActivities,
    EditTorrentUpDownFactors,
    StaffPm,
    DeleteTorrentReport,
    RemoveTitleGroupFromSeries,
    ManageTitleGroupTags,
    MergeTitleGroup,
    DeleteEditionGroup,
    MoveTorrentToOtherEditionGroup,
    ViewStatsDetails,
    DeleteUserEditChangeLog,
    LinkSimilarWikiArticles,
    GenerateResetPasswordToken,
    CreateUserBadge,
    DeleteUserBadge,
    ViewInvisibleUserBadges,
    AwardUserBadge,
    RevokeUserBadge,
    ReadAllConversationsThirdParty,
    ReadAllConversationsMember,
    ManageSiteHighlights,
    ManageRelatedForumThread,
    EditTorrent,
    EditTorrentAndTrumpable,
    LinkSimilarTitleGroup,
    UnlinkSimilarTitleGroup,
    SendMassPm,
    DeleteTitleGroupComment,
    DeleteTorrentRequestComment,
    DeleteTorrentRequest,
    SeeParanoiaHiddenUserInfo,
    SeeForeignBonusPointsLogs,
    ReactToContent,
    ViewForeignInvitations,
    WriteUserStaffNote,
    EditUserStaffNotes,
    ViewForeignUserStaffNotes,
    RequestReseed,
    ViewRegistrationIps,
    ManageAdventCalendars,
}

impl TestUser {
    fn get_login_payload(&self) -> Login {
        let username = match self {
            TestUser::Standard => "user_basic",
            TestUser::EditArtist => "user_edit_art",
            TestUser::EditSeries => "user_edit_ser",
            TestUser::EditTitleGroupComment => "user_edit_tgc",
            TestUser::CreateCssSheet => "user_css_crt",
            TestUser::EditCssSheet => "user_css_edit",
            TestUser::CreateForumCategory => "user_cat_crt",
            TestUser::EditForumCategory => "user_cat_edit",
            TestUser::CreateForumSubCategory => "user_sub_crt",
            TestUser::EditForumSubCategory => "user_sub_edit",
            TestUser::EditForumThread => "user_thr_edit",
            TestUser::EditForumPost => "user_post_edit",
            TestUser::DeleteForumCategory => "user_cat_del",
            TestUser::DeleteForumSubCategory => "user_sub_del",
            TestUser::DeleteForumThread => "user_thr_del",
            TestUser::DeleteForumPost => "user_post_del",
            TestUser::ForumCategoryFlow => "user_cat_flow",
            TestUser::ForumSubCategoryFlow => "user_sub_flow",
            TestUser::CreateUserClass => "user_cls_crt",
            TestUser::EditUserClass => "user_cls_edit",
            TestUser::DeleteUserClass => "user_cls_del",
            TestUser::SetAndViewUserPermissions => "user_perm_edit",
            TestUser::LockUserClass => "user_lock_cls",
            TestUser::ChangeUserClass => "user_cls_chg",
            TestUser::EditArcadiaSettings => "user_arc_set",
            TestUser::CreateDonation => "user_don_crt",
            TestUser::EditDonation => "user_don_edit",
            TestUser::DeleteDonation => "user_don_del",
            TestUser::SearchDonation => "user_don_srch",
            TestUser::SearchUsers => "user_usr_srch",
            TestUser::SearchUsersWithPermissions => "user_usr_perm",
            TestUser::SendMassPmWithPermissions => "user_mass_perm",
            TestUser::WarnBanUser => "user_warn_ban",
            TestUser::SearchUnauthorizedAccess => "user_unauth",
            TestUser::DeleteArtist => "user_art_del",
            TestUser::SetTorrentStaffChecked => "user_tor_stfc",
            TestUser::SearchUserEditChangeLogs => "user_edit_log",
            TestUser::EditEditionGroup => "user_edit_eg",
            TestUser::EditCollage => "user_col_edit",
            TestUser::DeleteCollage => "user_col_del",
            TestUser::DeleteCollageEntry => "user_ce_del",
            TestUser::DeleteTitleGroup => "user_tg_del",
            TestUser::ViewTorrentPeersAndActivities => "user_view_peers",
            TestUser::EditTorrentUpDownFactors => "user_tor_fact",
            TestUser::StaffPm => "user_staff_pm",
            TestUser::DeleteTorrentReport => "user_tr_del",
            TestUser::RemoveTitleGroupFromSeries => "user_rm_tg_ser",
            TestUser::ManageTitleGroupTags => "user_tag_manage",
            TestUser::MergeTitleGroup => "user_tg_merge",
            TestUser::DeleteEditionGroup => "user_eg_del",
            TestUser::MoveTorrentToOtherEditionGroup => "user_tor_move",
            TestUser::ViewStatsDetails => "user_view_stats",
            TestUser::DeleteUserEditChangeLog => "user_del_edlog",
            TestUser::LinkSimilarWikiArticles => "user_link_wiki",
            TestUser::GenerateResetPasswordToken => "user_pwd_chg",
            TestUser::CreateUserBadge => "user_badge_crt",
            TestUser::DeleteUserBadge => "user_badge_del",
            TestUser::ViewInvisibleUserBadges => "user_badge_view",
            TestUser::AwardUserBadge => "user_badge_awd",
            TestUser::RevokeUserBadge => "user_badge_rev",
            TestUser::ReadAllConversationsThirdParty => "user_ra_third",
            TestUser::ReadAllConversationsMember => "user_ra_membr",
            TestUser::ManageSiteHighlights => "user_site_high",
            TestUser::ManageRelatedForumThread => "user_rel_thr",
            TestUser::EditTorrent => "user_edit_tor",
            TestUser::EditTorrentAndTrumpable => "user_edit_trump",
            TestUser::LinkSimilarTitleGroup => "user_lnk_sim_tg",
            TestUser::UnlinkSimilarTitleGroup => "user_rm_sim_tg",
            TestUser::SendMassPm => "user_mass_pm",
            TestUser::DeleteTitleGroupComment => "user_del_tgc",
            TestUser::DeleteTorrentRequestComment => "user_del_trc",
            TestUser::DeleteTorrentRequest => "user_del_tr",
            TestUser::SeeParanoiaHiddenUserInfo => "user_paranoia",
            TestUser::SeeForeignBonusPointsLogs => "user_foreign_bp",
            TestUser::ReactToContent => "user_reactor",
            TestUser::ViewForeignInvitations => "user_frgn_inv",
            TestUser::WriteUserStaffNote => "user_wr_note",
            TestUser::EditUserStaffNotes => "user_ed_notes",
            TestUser::ViewForeignUserStaffNotes => "user_frgn_notes",
            TestUser::RequestReseed => "user_reseed",
            TestUser::ViewRegistrationIps => "user_reg_ips",
            TestUser::ManageAdventCalendars => "user_advent_cal",
        };

        Login {
            username: username.into(),
            password: "test_password".into(),
            remember_me: true,
        }
    }
}

// Requires "with_test_users" fixture.
pub async fn create_test_app_and_login<R: RedisPoolInterface + 'static>(
    pool: Arc<ConnectionPool>,
    redis_pool: R,
    test_user: TestUser,
) -> (
    impl Service<Request, Response = ServiceResponse, Error = Error>,
    LoginResponse,
) {
    let service = create_test_app(pool, redis_pool).await;
    let user = login_as(&service, test_user).await;
    (service, user)
}

// Requires "with_test_users" fixture. Logs an additional user in on an existing service.
pub async fn login_as<S>(service: &S, test_user: TestUser) -> LoginResponse
where
    S: Service<Request, Response = ServiceResponse, Error = Error>,
{
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(test_user.get_login_payload())
        .to_request();

    let user = call_and_read_body_json::<LoginResponse, _>(service, req).await;

    assert!(!user.token.is_empty());
    assert!(!user.refresh_token.is_empty());

    user
}

pub fn auth_header(token: &str) -> impl TryIntoHeaderPair {
    (AUTHORIZATION, format!("Bearer {}", token))
}

pub async fn read_body_json_data<T: DeserializeOwned, B: MessageBody>(
    resp: ServiceResponse<B>,
) -> T {
    let body = test::read_body(resp).await;
    let wrapper: serde_json::Value = serde_json::from_slice(&body).expect("valid JSON response");
    serde_json::from_value(wrapper["data"].clone()).expect("valid data field in response wrapper")
}

pub async fn read_body_bencode<T: DeserializeOwned, B: MessageBody>(
    resp: ServiceResponse<B>,
) -> Result<T, serde_bencode::Error> {
    let body = test::read_body(resp).await;
    serde_bencode::from_bytes(&body)
}

pub async fn call_and_read_body_json_with_status<T, S>(
    service: &S,
    req: Request,
    status_code: StatusCode,
) -> T
where
    S: Service<Request, Response = ServiceResponse, Error = Error>,
    T: DeserializeOwned,
{
    let resp = test::call_service(&service, req).await;

    assert_eq!(
        resp.status(),
        status_code,
        "expected HTTP status {}, got {}",
        status_code,
        resp.status()
    );

    let content_type = resp.headers().get(CONTENT_TYPE);

    assert_eq!(
        content_type,
        Some(&HeaderValue::from_static("application/json")),
        "expected Content-Type: application/json, got {content_type:?}"
    );

    let body = test::read_body(resp).await;
    let wrapper: serde_json::Value = serde_json::from_slice(&body).expect("valid JSON response");

    if status_code.is_success() {
        serde_json::from_value(wrapper["data"].clone())
            .expect("valid data field in response wrapper")
    } else {
        serde_json::from_value(wrapper).expect("valid JSON error response")
    }
}

#[inline]
pub async fn call_and_read_body_json<T, S>(service: &S, req: Request) -> T
where
    S: Service<Request, Response = ServiceResponse, Error = Error>,
    T: DeserializeOwned,
{
    call_and_read_body_json_with_status(service, req, StatusCode::OK).await
}
