#[derive(Debug, thiserror::Error, strum::AsRefStr)]
pub enum Error {
    #[error("database error: {0}")]
    GenericDatabaseError(#[from] sqlx::Error),

    #[error("{0}")]
    InvalidPassword(String),

    #[error("passwords do not match")]
    PasswordsDoNotMatch,

    #[error("donation amount must be positive")]
    DonationAmountMustBePositive,

    #[error("{0}")]
    InvalidArcadiaSettings(String),

    #[error("{0}")]
    BonusPointsSnatchCostOutOfRange(String),

    #[error("{0}")]
    InvalidTorrentSearchQuery(String),

    #[error("invalid bonus points formula: {0}")]
    InvalidBonusPointsFormula(String),

    #[error("{0}")]
    PromotionNotAvailable(String),

    #[error("{0}")]
    InvalidTagExpression(String),

    #[error("tag '{0}' was deleted: {1}")]
    TitleGroupTagDeleted(String, String),

    #[error("the name or one of the synonyms of this tag is already used by the tag '{0}'")]
    TitleGroupTagConflict(String),

    #[error("a tag cannot be merged into itself")]
    TitleGroupTagCannotBeMergedIntoItself,

    #[error("account banned")]
    AccountBanned,

    #[error("could not create user application")]
    CouldNotCreateUserApplication(#[source] sqlx::Error),

    #[error("could not get user applications")]
    CouldNotGetUserApplications(#[source] sqlx::Error),

    #[error("could not update user application")]
    CouldNotUpdateUserApplication(#[source] sqlx::Error),

    #[error("could not create artist")]
    CouldNotCreateArtist(#[source] sqlx::Error),

    #[error("could not update artist")]
    CouldNotUpdateArtist(#[source] sqlx::Error),

    #[error("could not delete artist")]
    CouldNotDeleteArtist(#[source] sqlx::Error),

    #[error("could not find artist")]
    CouldNotFindArtist(#[source] sqlx::Error),

    #[error("could not create artist affiliation")]
    CouldNotCreateArtistAffiliation(#[source] sqlx::Error),

    #[error("artist is already affiliated to this title group")]
    DuplicateArtistAffiliation,

    #[error("could not search for artists")]
    CouldNotSearchForArtists(#[source] sqlx::Error),

    #[error("could not search for users")]
    CouldNotSearchForUsers(#[source] sqlx::Error),

    #[error("could not create user")]
    CouldNotCreateUser(#[source] sqlx::Error),

    #[error("username already exists")]
    UsernameAlreadyExists,

    #[error("could not deserialize forum posts: {0}")]
    CouldNotDeserializeForumPosts(String),

    #[error("could not create edition group: '{0}'")]
    CouldNotCreateEditionGroup(#[source] sqlx::Error),

    #[error("edition group not found")]
    EditionGroupNotFound,

    #[error("error while updating edition_group: '{0}'")]
    ErrorWhileUpdatingEditionGroup(String),

    #[error("edition group has undeleted torrents and cannot be deleted")]
    EditionGroupHasUndeletedTorrents,

    #[error("you can only delete your own edition groups within 24 hours of creation")]
    EditionGroupDeletionWindowExpired,

    #[error("could not create invitation")]
    CouldNotCreateInvitation(#[source] sqlx::Error),

    #[error("could not create master group")]
    CouldNotCreateMasterGroup(#[source] sqlx::Error),

    #[error("could not create notification")]
    CouldNotCreateNotification(#[source] sqlx::Error),

    #[error("could not get unread notifications")]
    CouldNotGetUnreadNotifications(#[source] sqlx::Error),

    #[error("could not mark notification as read")]
    CouldNotMarkNotificationAsRead(#[source] sqlx::Error),

    #[error("could not create subscription")]
    CouldNotCreateSubscription(#[source] sqlx::Error),

    #[error("could not create title group subscription")]
    CouldNotCreateTitleGroupComment(#[source] sqlx::Error),

    #[error("could not create title group: '{0}'")]
    CouldNotCreateTitleGroup(#[source] sqlx::Error),

    #[error("could not create title group tag")]
    CouldNotCreateTitleGroupTag(#[source] sqlx::Error),

    #[error("title group tag not found")]
    TitleGroupTagNotFound,

    #[error("could not update title group tag")]
    CouldNotUpdateTitleGroupTag(#[source] sqlx::Error),

    #[error("could not delete title group tag")]
    CouldNotDeleteTitleGroupTag(#[source] sqlx::Error),

    #[error("could not create torrent: '{0}'")]
    CouldNotCreateTorrent(#[source] sqlx::Error),

    #[error("content released after {0} is not allowed")]
    ContentReleasedAfterCutoff(String),

    #[error("could not create torrent request")]
    CouldNotCreateTorrentRequest(#[source] sqlx::Error),

    #[error("could not search for torrent requests")]
    CouldNotSearchForTorrentRequests(#[source] sqlx::Error),

    #[error("could not find the torrent request")]
    CouldNotFindTheTorrentRequest(#[source] sqlx::Error),

    #[error("this torrent isn't in the title group of the request")]
    TorrentTitleGroupNotMatchingRequestedOne,

    #[error("this torrent request is already filled")]
    TorrentRequestAlreadyFilled,

    #[error(
        "during the first hour after upload, only the torrent uploader can fill requests with this torrent"
    )]
    TorrentRequestFillUploaderOnlyWithinFirstHour,

    #[error("could not create torrent request vote")]
    CouldNotCreateTorrentRequestVote(#[source] sqlx::Error),

    #[error("could not create torrent request comment")]
    CouldNotCreateTorrentRequestComment(#[source] sqlx::Error),

    #[error("could not create torrent report")]
    CouldNotCreateTorrentReport(#[source] sqlx::Error),

    #[error("could not delete torrent report")]
    CouldNotDeleteTorrentReport(#[source] sqlx::Error),

    #[error("could not get torrent report")]
    CouldNotGetTorrentReport(#[source] sqlx::Error),

    #[error("could not create series")]
    CouldNotCreateSeries(#[source] sqlx::Error),

    #[error("could not update series")]
    CouldNotUpdateSeries(#[source] sqlx::Error),

    #[error("could not delete series")]
    CouldNotDeleteSeries(#[source] sqlx::Error),

    #[error("could not create api key")]
    CouldNotCreateAPIKey(#[source] sqlx::Error),

    #[error("api key not found")]
    APIKeyNotFound,

    #[error("you cannot have more than {0} api keys")]
    TooManyAPIKeys(i64),

    #[error("this api key is not allowed to reach this endpoint")]
    APIKeyScopeNotAllowed,

    #[error("series with id '{0}' not found")]
    SeriesWithIdNotFound(i64),

    #[error("invalid invitation key")]
    InvitationKeyInvalid,

    #[error("email configuration error: {0}")]
    EmailConfigurationError(String),

    #[error("failed to send email: {0}")]
    EmailSendError(String),

    #[error("IRC configuration error: {0}")]
    IrcConfigurationError(String),

    #[error("failed to provision IRC account: {0}")]
    IrcAccountProvisioningError(String),

    #[error("IRC integration is not enabled")]
    IrcNotEnabled,

    #[error("too many notification streams are open")]
    TooManyNotificationStreams,

    #[error("IRC account already exists")]
    IrcAccountAlreadyExists,

    #[error("IRC account not found")]
    IrcAccountNotFound,

    #[error("invitation key required")]
    InvitationKeyRequired,

    #[error("invitation key already used")]
    InvitationKeyAlreadyUsed,

    #[error("no invitations available")]
    NoInvitationsAvailable,

    #[error("user '{0}' not found")]
    UserNotFound(String),

    #[error("user with id '{0}' not found")]
    UserWithIdNotFound(i32),

    #[error("wrong username or password")]
    WrongUsernameOrPassword,

    #[error("invalid API key or banned")]
    InvalidAPIKeyOrBanned,

    #[error("invalid or expired refresh token")]
    InvalidOrExpiredRefreshToken,

    #[error("invalid or expired password reset token")]
    InvalidOrExpiredPasswordResetToken,

    #[error("invalided token")]
    InvalidatedToken,

    #[error("JWT error")]
    JwtError(#[source] jsonwebtoken::errors::Error),

    #[error("unsupported notification reason")]
    UnsupportedNotification,

    #[error("unsupported subscription type '{0}'")]
    UnsupportedSubscription(String),

    #[error("not enough bonus points to place this bounty")]
    InsufficientBonusPointsForBounty,

    #[error("not enough upload to place this bounty")]
    InsufficientUploadForBounty,

    #[error("vote bounty must be positive in at least one enabled currency")]
    VoteBountyRequired,

    #[error("torrent file invalid")]
    TorrentFileInvalid,

    #[error("torrent is too large, the maximum size is {0} bytes")]
    TorrentTooLarge(u64),

    #[error("dottorrent file not found")]
    DottorrentFileNotFound,

    #[error("torrent not found")]
    TorrentNotFound,

    #[error("torrent request not found")]
    TorrentRequestNotFound,

    #[error("error while updating torrent_request: '{0}'")]
    ErrorWhileUpdatingTorrentRequest(String),

    #[error("error while deleting torrent request: '{0}'")]
    ErrorWhileDeletingTorrentRequest(String),

    #[error("this torrent request has votes from other users and cannot be deleted")]
    TorrentRequestHasVotesFromOtherUsers,

    #[error("the title group of a torrent request cannot change once it is filled or has votes from other users")]
    TorrentRequestTitleGroupLocked,

    #[error("title group not found")]
    TitleGroupNotFound,

    #[error("title group has undeleted torrents and cannot be deleted")]
    TitleGroupHasUndeletedTorrents,

    #[error("cannot merge a title group into itself")]
    CannotMergeTitleGroupIntoItself,

    #[error("cannot merge title groups with different content types")]
    CannotMergeTitleGroupsWithDifferentContentTypes,

    #[error("error while updating title_group: '{0}'")]
    ErrorWhileUpdatingTitleGroup(String),

    #[error("could not find title group comment")]
    CouldNotFindTitleGroupComment(#[source] sqlx::Error),

    #[error("error while updating title_group_comment: '{0}'")]
    ErrorWhileUpdatingTitleGroupComment(String),

    #[error("could not delete title group comment")]
    CouldNotDeleteTitleGroupComment(#[source] sqlx::Error),

    #[error("could not search title group comments")]
    CouldNotSearchTitleGroupComments(#[source] sqlx::Error),

    #[error("could not find torrent request comment")]
    CouldNotFindTorrentRequestComment(#[source] sqlx::Error),

    #[error("could not delete torrent request comment")]
    CouldNotDeleteTorrentRequestComment(#[source] sqlx::Error),

    #[error("could not search torrent request comments")]
    CouldNotSearchTorrentRequestComments(#[source] sqlx::Error),

    #[error("edition groups are not in the same title group")]
    EditionGroupsNotInSameTitleGroup,

    #[error("error while updating torrent: '{0}'")]
    ErrorWhileUpdatingTorrent(String),

    #[error("could not save torrent file in path: '{0}'\n'{1}'")]
    CouldNotSaveTorrentFile(String, String),

    #[error("error while searching for torrents: '{0}'")]
    ErrorSearchingForTorrents(String),

    #[error("error while searching for title group: '{0}'")]
    ErrorSearchingForTitleGroup(String),

    #[error("error while deleting torrent: '{0}'")]
    ErrorDeletingTorrent(String),

    #[error("unexpected third party response")]
    UnexpectedThirdPartyResponse(#[from] reqwest::Error),

    #[error("not enough bonus points available")]
    NotEnoughBonusPointsAvailable,

    #[error("not enough freeleech tokens available")]
    NotEnoughFreeleechTokensAvailable,

    #[error("could not create gift")]
    CouldNotCreateGift(#[source] sqlx::Error),

    #[error("could not create forum post")]
    CouldNotCreateForumPost(#[source] sqlx::Error),

    #[error("could not update forum post")]
    CouldNotUpdateForumPost(#[source] sqlx::Error),

    #[error("could not update forum thread")]
    CouldNotUpdateForumThread(#[source] sqlx::Error),

    #[error("forum thread locked")]
    ForumThreadLocked,

    #[error("staff PM is resolved")]
    StaffPmResolved,

    #[error("forum thread name cannot be empty")]
    ForumThreadNameEmpty,

    #[error("forum post empty")]
    ForumPostEmpty,

    #[error("forum poll question cannot be empty")]
    ForumPollQuestionEmpty,

    #[error("forum poll must have at least one option")]
    ForumPollOptionsInsufficient,

    #[error("forum thread already has a poll")]
    ForumThreadAlreadyHasPoll,

    #[error("forum poll option not found")]
    ForumPollOptionNotFound,

    #[error("user has already voted on this poll")]
    ForumPollAlreadyVoted,

    #[error("could not create forum poll")]
    CouldNotCreateForumPoll(#[source] sqlx::Error),

    #[error("could not create forum poll vote")]
    CouldNotCreateForumPollVote(#[source] sqlx::Error),

    #[error("could not find forum poll")]
    CouldNotFindForumPoll(#[source] sqlx::Error),

    #[error("could not find forum post")]
    CouldNotFindForumPost(#[source] sqlx::Error),

    #[error("could not create forum thread")]
    CouldNotCreateForumThread(#[source] sqlx::Error),

    #[error("could not find forum sub-category")]
    CouldNotFindForumSubCategory(#[source] sqlx::Error),

    #[error("could not find forum thread")]
    CouldNotFindForumThread(#[source] sqlx::Error),

    #[error("could not find first posts in threads of forum sub category")]
    CouldNotFindForumThreadsFirstPost(#[source] sqlx::Error),

    #[error("could not search forum threads")]
    CouldNotSearchForumThreads(#[source] sqlx::Error),

    #[error("could not search forum posts")]
    CouldNotSearchForumPosts(#[source] sqlx::Error),

    #[error("could not create forum category")]
    CouldNotCreateForumCategory(#[source] sqlx::Error),

    #[error("could not pin/unpin forum thread")]
    CouldNotPinForumThread(#[source] sqlx::Error),

    #[error("could not update forum category")]
    CouldNotUpdateForumCategory(#[source] sqlx::Error),

    #[error("forum category not found")]
    ForumCategoryNotFound,

    #[error("forum category name cannot be empty")]
    ForumCategoryNameEmpty,

    #[error("could not create forum sub-category")]
    CouldNotCreateForumSubCategory(#[source] sqlx::Error),

    #[error("could not update forum sub-category")]
    CouldNotUpdateForumSubCategory(#[source] sqlx::Error),

    #[error("forum sub-category not found")]
    ForumSubCategoryNotFound,

    #[error("forum sub-category name cannot be empty")]
    ForumSubCategoryNameEmpty,

    #[error("could not delete forum category")]
    CouldNotDeleteForumCategory(#[source] sqlx::Error),

    #[error("forum category has sub-categories and cannot be deleted")]
    ForumCategoryHasSubCategories,

    #[error("could not delete forum sub-category")]
    CouldNotDeleteForumSubCategory(#[source] sqlx::Error),

    #[error("forum sub-category has threads and cannot be deleted")]
    ForumSubCategoryHasThreads,

    #[error("forum sub-category thread creation is restricted")]
    ForumSubCategoryNewThreadsRestricted,

    #[error("could not reorder forum category")]
    CouldNotReorderForumCategory(#[source] sqlx::Error),

    #[error("could not reorder forum sub-category")]
    CouldNotReorderForumSubCategory(#[source] sqlx::Error),

    #[error("could not delete forum thread")]
    CouldNotDeleteForumThread(#[source] sqlx::Error),

    #[error("could not delete forum post")]
    CouldNotDeleteForumPost(#[source] sqlx::Error),

    #[error("could not upsert forum thread read")]
    CouldNotUpsertForumThreadRead(#[source] sqlx::Error),

    #[error("insufficient permissions: missing {0}")]
    InsufficientPermissions(String),

    #[error("you can only delete your own torrents within 24 hours of uploading")]
    TorrentDeletionWindowExpired,

    #[error("reseed request not allowed: {0}")]
    ReseedNotEligible(String),

    #[error("you have already sent a reseed request for this torrent")]
    ReseedAlreadyRequested,

    #[error("you can only move your own torrents within 24 hours of uploading")]
    TorrentMoveWindowExpired,

    #[error("could not warn user: '{0}'")]
    CouldNotWarnUser(String),

    #[error("invalid user id or torrent id")]
    InvalidUserIdOrTorrentId,

    #[error("could not create wiki article")]
    CouldNotCreateWikiArticle(#[source] sqlx::Error),

    #[error("could not find wiki article")]
    CouldNotFindWikiArticle(#[source] sqlx::Error),

    #[error("a wiki article cannot be linked to itself")]
    WikiArticleCannotBeLinkedToItself,

    #[error("could not link similar wiki articles")]
    CouldNotLinkSimilarWikiArticles(#[source] sqlx::Error),

    #[error("could not unlink similar wiki articles")]
    CouldNotUnlinkSimilarWikiArticles(#[source] sqlx::Error),

    #[error("a title group cannot be linked to itself")]
    TitleGroupCannotBeLinkedToItself,

    #[error("could not link similar title groups")]
    CouldNotLinkSimilarTitleGroups(#[source] sqlx::Error),

    #[error("could not unlink similar title groups")]
    CouldNotUnlinkSimilarTitleGroups(#[source] sqlx::Error),

    #[error("could not create bookmark")]
    CouldNotCreateTitleGroupBookmark(#[source] sqlx::Error),

    #[error("could not find bookmark")]
    CouldNotFindTitleGroupBookmark(#[source] sqlx::Error),

    #[error("error while updating bookmark: '{0}'")]
    ErrorWhileUpdatingTitleGroupBookmark(String),

    #[error("could not create conversation")]
    CouldNotCreateConversation(#[source] sqlx::Error),

    #[error("could not create message")]
    CouldNotCreateConversationMessage(#[source] sqlx::Error),

    #[error("could not find conversation")]
    CouldNotFindConversation(#[source] sqlx::Error),

    #[error("could not find conversations")]
    CouldNotFindConversations(#[source] sqlx::Error),

    #[error("could not search conversations")]
    CouldNotSearchConversations(#[source] sqlx::Error),

    #[error("conversation is locked")]
    ConversationLocked,

    #[error("could not create collage")]
    CouldNotCreateCollage(#[source] sqlx::Error),

    #[error("could not create collage entry: {0}")]
    CouldNotCreateCollageEntry(String),

    #[error("could not fetch collage")]
    CouldNotFetchCollage(#[source] sqlx::Error),

    #[error("could not update collage")]
    CouldNotUpdateCollage(#[source] sqlx::Error),

    #[error("could not delete collage")]
    CouldNotDeleteCollage(#[source] sqlx::Error),

    #[error("collage has entries and cannot be deleted")]
    CollageHasEntries,

    #[error("could not delete collage entry")]
    CouldNotDeleteCollageEntry(#[source] sqlx::Error),

    #[error("could not create css sheet")]
    CouldNotCreateCssSheet(#[source] sqlx::Error),

    #[error("css sheet not found")]
    CssSheetNotFound(#[source] sqlx::Error),

    #[error("could not update default css sheet")]
    CouldNotUpdateDefaultCssSheet(#[source] sqlx::Error),

    #[error("could not find css sheets")]
    CouldNotFindCssSheets(#[source] sqlx::Error),

    #[error("could not find arcadia settings")]
    CouldNotFindArcadiaSettings(#[source] sqlx::Error),

    #[error("could not update arcadia settings")]
    CouldNotUpdateArcadiaSettings(#[source] sqlx::Error),

    #[error("error getting musicbrainz data")]
    ErrorGettingMusicbrainzData(#[source] Box<musicbrainz_rs::ApiEndpointError>),

    #[error("invalid email address")]
    InvalidEmailAddress,

    #[error("invalid username")]
    InvalidUsername,

    #[error("invalid musicbrainz url")]
    InvalidMusicbrainzUrl,

    #[error("invalid comic vine url")]
    InvalidComicVineUrl,

    #[error("comic vine data fetching not available")]
    ComicVineDataFetchingNotAvailable,

    #[error("tmdb data fetching not available")]
    TMDBDataFetchingNotAvailable,

    #[error("error fetching tmdb data")]
    TMDBDataFetchingError,

    #[error("invalid tmdb url")]
    InvalidTMDBUrl,

    #[error("external source '{0}' not found")]
    ExternalSourceNotFound(String),

    #[error("error getting data from the external source '{0}'")]
    ExternalSourcePluginError(String),

    /// The message an external source plugin answered with, written for the uploader and shown to
    /// them as is.
    #[error("{0}")]
    ExternalSourcePluginMessage(String),

    #[error("redis error '{0}'")]
    RedisError(String),

    #[error("serde error")]
    SerdeError(#[from] serde_json::Error),

    #[error("user class '{0}' not found")]
    UserClassNotFound(String),

    #[error("user class already exists")]
    UserClassAlreadyExists,

    #[error("user class is locked and cannot be modified")]
    UserClassLocked,

    #[error("invalid user class name")]
    InvalidUserClassName,

    #[error("could not create user class")]
    CouldNotCreateUserClass(#[source] sqlx::Error),

    #[error("could not update user class")]
    CouldNotUpdateUserClass(#[source] sqlx::Error),

    #[error("could not delete user class")]
    CouldNotDeleteUserClass(#[source] sqlx::Error),

    #[error("could not fetch donations")]
    CouldNotFetchDonations(#[source] sqlx::Error),

    #[error("could not fetch donation")]
    CouldNotFetchDonation(#[source] sqlx::Error),

    #[error("could not create donation")]
    CouldNotCreateDonation(#[source] sqlx::Error),

    #[error("could not update donation")]
    CouldNotUpdateDonation(#[source] sqlx::Error),

    #[error("could not delete donation")]
    CouldNotDeleteDonation(#[source] sqlx::Error),

    #[error("could not create user edit change log")]
    CouldNotCreateUserEditChangeLog(#[source] sqlx::Error),

    #[error("could not delete user edit change log")]
    CouldNotDeleteUserEditChangeLog(#[source] sqlx::Error),

    #[error("could not delete all user edit change logs")]
    CouldNotDeleteAllUserEditChangeLogs(#[source] sqlx::Error),

    #[error("image host not approved: {url}. Approved hosts: {}", approved_hosts.join(", "))]
    ImageHostNotApproved {
        url: String,
        approved_hosts: Vec<String>,
    },

    #[error("image host not configured")]
    ImageHostNotConfigured,

    #[error("invalid trailer url: {0}, only http(s) links are allowed")]
    InvalidTrailerUrl(String),

    #[error("image host upload failed: {0}")]
    ImageHostUploadFailed(String),

    #[error("invalid shop purchase amount")]
    InvalidShopPurchaseAmount,

    #[error("shop item not available")]
    ShopItemNotAvailable,

    #[error("uploads are temporarily disabled")]
    UploadsDisabled,

    #[error("could not create user badge category")]
    CouldNotCreateUserBadgeCategory(#[source] sqlx::Error),

    #[error("could not update user badge category")]
    CouldNotUpdateUserBadgeCategory(#[source] sqlx::Error),

    #[error("could not delete user badge category")]
    CouldNotDeleteUserBadgeCategory(#[source] sqlx::Error),

    #[error("could not find user badge categories")]
    CouldNotFindUserBadgeCategories(#[source] sqlx::Error),

    #[error("user badge category not found")]
    UserBadgeCategoryNotFound,

    #[error("user badge category name cannot be empty")]
    UserBadgeCategoryNameEmpty,

    #[error("user badge category has badges and cannot be deleted")]
    UserBadgeCategoryHasBadges,

    #[error("could not create user badge")]
    CouldNotCreateUserBadge(#[source] sqlx::Error),

    #[error("could not update user badge")]
    CouldNotUpdateUserBadge(#[source] sqlx::Error),

    #[error("could not delete user badge")]
    CouldNotDeleteUserBadge(#[source] sqlx::Error),

    #[error("could not find user badges")]
    CouldNotFindUserBadges(#[source] sqlx::Error),

    #[error("user badge not found")]
    UserBadgeNotFound,

    #[error("user badge name cannot be empty")]
    UserBadgeNameEmpty,

    #[error("user badge criteria mismatch with badge type")]
    UserBadgeCriteriaMismatch,

    #[error("could not award user badge")]
    CouldNotAwardUserBadge(#[source] sqlx::Error),

    #[error("could not revoke user badge")]
    CouldNotRevokeUserBadge(#[source] sqlx::Error),

    #[error("user already has this badge")]
    UserAlreadyHasBadge,

    #[error("user earned badge not found")]
    UserEarnedBadgeNotFound,

    #[error("could not find user staff notes")]
    CouldNotFindUserStaffNotes(#[source] sqlx::Error),

    #[error("could not create user staff note")]
    CouldNotCreateUserStaffNote(#[source] sqlx::Error),

    #[error("could not edit user staff note")]
    CouldNotEditUserStaffNote(#[source] sqlx::Error),

    #[error("could not delete user staff note")]
    CouldNotDeleteUserStaffNote(#[source] sqlx::Error),

    #[error("user staff note not found")]
    UserStaffNoteNotFound,

    #[error("no ip check provider could check the ip address")]
    IpCheckFailed,

    #[error("only public ip addresses can be checked")]
    IpNotPublic,

    #[error("could not create site highlight")]
    CouldNotCreateSiteHighlight(#[source] sqlx::Error),

    #[error("could not edit site highlight")]
    CouldNotEditSiteHighlight(#[source] sqlx::Error),

    #[error("could not delete site highlight")]
    CouldNotDeleteSiteHighlight(#[source] sqlx::Error),

    #[error("could not get site highlights")]
    CouldNotGetSiteHighlights(#[source] sqlx::Error),

    #[error("could not get related threads")]
    CouldNotGetRelatedThreads(#[source] sqlx::Error),

    #[error("could not delete related thread")]
    CouldNotDeleteRelatedThread(#[source] sqlx::Error),

    #[error("could not create related thread")]
    CouldNotCreateRelatedThread(#[source] sqlx::Error),

    #[error("site highlight not found")]
    SiteHighlightNotFound,

    #[error("related forum thread not found")]
    RelatedForumThreadNotFound,

    #[error("position already taken by another site highlight")]
    SiteHighlightPositionTaken,

    #[error("{0}")]
    InvalidSiteHighlight(String),

    #[error("the emoji image is larger than 32 KiB")]
    EmojiImageTooLarge,
    #[error("the emoji image must be a png, a webp, a gif or an svg")]
    InvalidEmojiImageMimeType,
    #[error("an emoji must have either a unicode character or an image, and not both")]
    EmojiMustHaveExactlyOneRepresentation,
    #[error("emoji not found")]
    EmojiNotFound,
    #[error("the emoji or the forum post does not exist")]
    EmojiOrForumPostNotFound,
    #[error("an emoji with this name already exists")]
    EmojiNameAlreadyExists,
    #[error("the emoji name must not be blank")]
    InvalidEmojiName,
    #[error("could not reorder emojis")]
    CouldNotReorderEmojis(#[source] sqlx::Error),
    #[error("this emoji is disabled")]
    EmojiDisabled,
    #[error("could not get the uploaded torrents")]
    CouldNotGetUploadedTorrents(#[source] sqlx::Error),
    #[error("could not get the snatched torrents")]
    CouldNotGetSnatchedTorrents(#[source] sqlx::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl actix_web::ResponseError for Error {
    #[inline]
    fn status_code(&self) -> actix_web::http::StatusCode {
        use actix_web::http::StatusCode;

        match self {
            // 400 Bad Request
            Error::UsernameAlreadyExists
            | Error::InvitationKeyInvalid
            | Error::InvitationKeyRequired
            | Error::InvitationKeyAlreadyUsed
            | Error::WrongUsernameOrPassword
            | Error::InvalidOrExpiredPasswordResetToken
            | Error::TorrentFileInvalid
            | Error::TorrentTooLarge(_)
            | Error::InvalidUserIdOrTorrentId
            | Error::ForumThreadNameEmpty
            | Error::ForumPollQuestionEmpty
            | Error::ForumPollOptionsInsufficient
            | Error::ForumPostEmpty
            | Error::ForumCategoryNameEmpty
            | Error::ForumSubCategoryNameEmpty
            | Error::ForumCategoryHasSubCategories
            | Error::ForumSubCategoryHasThreads
            | Error::CollageHasEntries
            | Error::TitleGroupHasUndeletedTorrents
            | Error::EditionGroupHasUndeletedTorrents
            | Error::CannotMergeTitleGroupIntoItself
            | Error::CannotMergeTitleGroupsWithDifferentContentTypes
            | Error::InvalidUserClassName
            | Error::ImageHostNotApproved { .. }
            | Error::ImageHostNotConfigured
            | Error::InvalidTrailerUrl(_)
            | Error::ContentReleasedAfterCutoff(_)
            | Error::VoteBountyRequired
            | Error::InvalidPassword(_)
            | Error::PasswordsDoNotMatch
            | Error::DonationAmountMustBePositive
            | Error::InvalidArcadiaSettings(_)
            | Error::BonusPointsSnatchCostOutOfRange(_)
            | Error::InvalidTorrentSearchQuery(_)
            | Error::InvalidBonusPointsFormula(_)
            | Error::PromotionNotAvailable(_)
            | Error::InvalidTagExpression(_)
            | Error::TitleGroupTagDeleted(..)
            | Error::TitleGroupTagConflict(_)
            | Error::TitleGroupTagCannotBeMergedIntoItself
            | Error::EditionGroupsNotInSameTitleGroup
            | Error::UserBadgeCategoryNameEmpty
            | Error::UserBadgeCategoryHasBadges
            | Error::UserBadgeNameEmpty
            | Error::UserBadgeCriteriaMismatch
            | Error::WikiArticleCannotBeLinkedToItself
            | Error::TitleGroupCannotBeLinkedToItself
            | Error::InvalidSiteHighlight(_)
            | Error::TooManyAPIKeys(_)
            | Error::EmojiImageTooLarge
            | Error::InvalidEmojiImageMimeType
            | Error::EmojiMustHaveExactlyOneRepresentation
            | Error::EmojiNameAlreadyExists
            | Error::InvalidEmojiName
            | Error::EmojiDisabled
            | Error::IpNotPublic => StatusCode::BAD_REQUEST,

            // 401 Unauthorized
            Error::InvalidOrExpiredRefreshToken | Error::InvalidatedToken => {
                StatusCode::UNAUTHORIZED
            }

            // 403 Forbidden
            Error::AccountBanned
            | Error::InsufficientPermissions(_)
            | Error::TorrentDeletionWindowExpired
            | Error::TorrentRequestHasVotesFromOtherUsers
            | Error::TorrentRequestTitleGroupLocked
            | Error::TorrentMoveWindowExpired
            | Error::EditionGroupDeletionWindowExpired
            | Error::ForumThreadLocked
            | Error::ForumSubCategoryNewThreadsRestricted
            | Error::ConversationLocked
            | Error::StaffPmResolved
            | Error::UserClassLocked
            | Error::ShopItemNotAvailable
            | Error::UploadsDisabled
            | Error::APIKeyScopeNotAllowed => StatusCode::FORBIDDEN,

            // 404 Not Found
            Error::IrcAccountNotFound
            | Error::APIKeyNotFound
            | Error::UserNotFound(_)
            | Error::UserWithIdNotFound(_)
            | Error::SeriesWithIdNotFound(_)
            | Error::DottorrentFileNotFound
            | Error::TorrentNotFound
            | Error::TorrentRequestNotFound
            | Error::CouldNotFindArtist(_)
            | Error::TitleGroupTagNotFound
            | Error::CouldNotFindTitleGroupComment(_)
            | Error::CouldNotFindTorrentRequestComment(_)
            | Error::CouldNotFindForumThread(_)
            | Error::CouldNotFindForumSubCategory(_)
            | Error::CouldNotFindForumPost(_)
            | Error::CouldNotFindForumPoll(_)
            | Error::ForumPollOptionNotFound
            | Error::CssSheetNotFound(_)
            | Error::ForumCategoryNotFound
            | Error::ForumSubCategoryNotFound
            | Error::UserClassNotFound(_)
            | Error::UserBadgeCategoryNotFound
            | Error::UserBadgeNotFound
            | Error::UserEarnedBadgeNotFound
            | Error::UserStaffNoteNotFound
            | Error::EditionGroupNotFound
            | Error::SiteHighlightNotFound
            | Error::RelatedForumThreadNotFound
            | Error::ExternalSourceNotFound(_)
            | Error::EmojiNotFound
            | Error::EmojiOrForumPostNotFound => StatusCode::NOT_FOUND,

            // 409 Conflict
            Error::IrcAccountAlreadyExists
            | Error::NoInvitationsAvailable
            | Error::NotEnoughBonusPointsAvailable
            | Error::NotEnoughFreeleechTokensAvailable
            | Error::TorrentRequestAlreadyFilled
            | Error::TorrentTitleGroupNotMatchingRequestedOne
            | Error::TorrentRequestFillUploaderOnlyWithinFirstHour
            | Error::InsufficientBonusPointsForBounty
            | Error::InsufficientUploadForBounty
            | Error::UserClassAlreadyExists
            | Error::UserAlreadyHasBadge
            | Error::DuplicateArtistAffiliation
            | Error::ForumThreadAlreadyHasPoll
            | Error::ForumPollAlreadyVoted
            | Error::SiteHighlightPositionTaken
            | Error::ReseedNotEligible(_)
            | Error::ReseedAlreadyRequested => StatusCode::CONFLICT,

            // 502 Bad Gateway
            Error::ExternalSourcePluginError(_) | Error::ExternalSourcePluginMessage(_) => {
                StatusCode::BAD_GATEWAY
            }

            // 429 Too Many Requests
            Error::TooManyNotificationStreams => StatusCode::TOO_MANY_REQUESTS,

            // 503 Service Unavailable
            Error::IrcNotEnabled | Error::IpCheckFailed => StatusCode::SERVICE_UNAVAILABLE,

            // 500 Internal Server Error
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        let status_code = self.status_code();
        log::error!("The request generated this error: {self}");
        crate::metrics::record_error(self.as_ref(), status_code.as_u16());
        actix_web::HttpResponse::build(status_code).json(serde_json::json!({
            "error": format!("{self}"),
        }))
    }
}
