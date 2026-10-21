CREATE EXTENSION IF NOT EXISTS unaccent;

CREATE TYPE user_permissions_enum AS ENUM (
    'create_user_class',
    'edit_user_class',
    'delete_user_class',
    'set_and_view_user_permissions',
    'change_user_class',
    'lock_user_class',
    'upload_torrent',
    'download_torrent',
    'create_torrent_request',
    'immune_activity_pruning',
    'edit_title_group',
    'edit_title_group_comment',
    'edit_edition_group',
    'edit_torrent',
    'edit_artist',
    'delete_artist',
    'delete_title_group',
    'edit_collage',
    'delete_collage',
    'edit_series',
    'delete_series',
    'remove_title_group_from_series',
    'edit_torrent_request',
    'edit_forum_post',
    'edit_forum_thread',
    'pin_forum_thread',
    'lock_forum_thread',
    'edit_forum_sub_category',
    'edit_forum_category',
    'create_forum_category',
    'create_forum_sub_category',
    'create_forum_thread',
    'create_forum_post',
    'send_pm',
    'create_css_sheet',
    'edit_css_sheet',
    'read_staff_pm',
    'reply_staff_pm',
    'resolve_staff_pm',
    'unresolve_staff_pm',
    'manage_title_group_tags',
    'edit_title_group_tag',
    'delete_torrent',
    'request_reseed',
    'set_torrent_staff_checked',
    'get_user_application',
    'update_user_application',
    'warn_user',
    'ban_user',
    'remove_user_warning',
    'edit_user',
    'generate_reset_password_token',
    'create_wiki_article',
    'edit_wiki_article',
    'link_similar_wiki_articles',
    'edit_arcadia_settings',
    'create_donation',
    'edit_donation',
    'delete_donation',
    'search_donation',
    'search_users',
    'search_unauthorized_access',
    'search_user_edit_change_logs',
    'delete_user_edit_change_log',
    'delete_forum_category',
    'delete_forum_sub_category',
    'delete_forum_thread',
    'delete_forum_post',
    'delete_title_group_comment',
    'delete_torrent_request_comment',
    'delete_torrent_request',
    'view_torrent_peers',
    'edit_torrent_up_down_factors',
    'delete_collage_entry',
    'delete_torrent_report',
    'see_foreign_torrent_clients',
    'set_user_custom_title',
    'merge_title_group',
    'delete_edition_group',
    'move_torrent_to_other_edition_group',
    'view_stats_details',
    'read_all_conversations',
    'create_user_badge',
    'edit_user_badge',
    'delete_user_badge',
    'view_invisible_user_badges',
    'create_user_badge_category',
    'edit_user_badge_category',
    'delete_user_badge_category',
    'award_user_badge',
    'revoke_user_badge',
    'manage_site_highlights',
    'manage_related_forum_thread',
    'create_forum_poll_vote',
    'use_maintenance_tools',
    'edit_torrent_trumpable',
    'link_similar_title_group',
    'unlink_similar_title_group',
    'send_mass_pm',
    'see_paranoia_hidden_user_info',
    'see_foreign_bonus_points_logs',
    'react_to_content',
    'view_foreign_invitations',
    'write_user_staff_note',
    'edit_user_staff_notes',
    'view_foreign_user_staff_notes',
    'view_registration_ips'
);
CREATE TABLE user_classes (
    name VARCHAR(30) UNIQUE NOT NULL,
    -- given on promotion, removed on demotion
    -- those permissions are never checked when a user does an action
    -- it is only the ones in the users' table that are checked
    new_permissions user_permissions_enum[] NOT NULL DEFAULT '{}',
    -- same with this value. it is the one in the users' table that is checked
    max_snatches_per_day INT,
    automatic_promotion BOOLEAN NOT NULL DEFAULT TRUE,
    automatic_demotion BOOLEAN NOT NULL DEFAULT TRUE,
    promotion_allowed_while_warned BOOLEAN NOT NULL DEFAULT false,
    previous_user_class VARCHAR(30) REFERENCES user_classes(name) ON DELETE SET NULL DEFAULT NULL,

    required_account_age_in_days INT NOT NULL DEFAULT 0,
    required_ratio FLOAT NOT NULL DEFAULT 0,
    required_torrent_uploads INT NOT NULL DEFAULT 0,
    required_torrent_uploads_in_unique_title_groups INT NOT NULL DEFAULT 0,
    required_uploaded BIGINT NOT NULL DEFAULT 0,
    required_torrent_snatched INT NOT NULL DEFAULT 0,
    required_downloaded BIGINT NOT NULL DEFAULT 0,
    required_forum_posts INT NOT NULL DEFAULT 0,
    required_forum_posts_in_unique_threads INT NOT NULL DEFAULT 0,
    required_title_group_comments INT NOT NULL DEFAULT 0,
    required_seeding_size BIGINT NOT NULL DEFAULT 0,
    promotion_cost_bonus_points BIGINT NOT NULL DEFAULT 0
);
INSERT INTO user_classes (name, new_permissions)
VALUES ('newbie', '{}');
-- user statistics that can be displayed on a user's profile.
-- the site's staff chooses which ones are displayed site wide, and each user can additionally
-- hide some of them with their paranoia settings
CREATE TYPE displayable_user_stats_enum AS ENUM (
    'uploaded',
    'real_uploaded',
    'downloaded',
    'real_downloaded',
    'ratio',
    'title_groups',
    'edition_groups',
    'torrents',
    'forum_posts',
    'forum_threads',
    'title_group_comments',
    'request_comments',
    'artist_comments',
    'seeding',
    'leeching',
    'snatched',
    'seeding_size',
    'requests_filled',
    'collages_started',
    'requests_voted',
    'average_seeding_time',
    'invited',
    'invitations',
    'bonus_points',
    'freeleech_tokens',
    'current_streak',
    'highest_streak',
    'joined_at',
    'last_seen'
);
-- user information that is displayed as a list, and that can be hidden with the paranoia settings.
-- every value must also exist in displayable_user_stats_enum: a list can only be displayed
-- if the count it belongs to is displayed as well
CREATE TYPE displayable_user_lists_enum AS ENUM (
    'torrents',
    'snatched',
    'forum_posts',
    'title_group_comments',
    'request_comments'
);
CREATE TYPE ip_check_provider_enum AS ENUM (
    'ip_api',
    'proxycheck',
    'ipapi_is',
    'ipqualityscore',
    'vpnapi'
);
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(15) UNIQUE NOT NULL,
    avatar TEXT,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    registered_from_ip INET NOT NULL,
    -- what an ip check provider told about registered_from_ip on registration, every column is
    -- NULL when it could not be checked, and the ones the provider does not report stay NULL
    registration_ip_check_provider ip_check_provider_enum,
    -- ISO 3166-1 alpha-2
    registration_ip_country_code TEXT,
    registration_ip_country TEXT,
    registration_ip_region TEXT,
    registration_ip_city TEXT,
    registration_ip_latitude DOUBLE PRECISION,
    registration_ip_longitude DOUBLE PRECISION,
    registration_ip_asn TEXT,
    registration_ip_organization TEXT,
    registration_ip_is_vpn BOOLEAN,
    registration_ip_is_proxy BOOLEAN,
    registration_ip_is_tor BOOLEAN,
    registration_ip_is_hosting BOOLEAN,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    description TEXT NOT NULL DEFAULT '',
    uploaded BIGINT NOT NULL DEFAULT 0,
    real_uploaded BIGINT NOT NULL DEFAULT 0,
    -- 1 byte downloaded
    downloaded BIGINT NOT NULL DEFAULT 1,
    real_downloaded BIGINT NOT NULL DEFAULT 1,
    last_seen TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    class_name VARCHAR(30) NOT NULL REFERENCES user_classes(name) ON UPDATE CASCADE,
    class_locked BOOLEAN NOT NULL DEFAULT FALSE,
    permissions user_permissions_enum[] NOT NULL DEFAULT '{}',
    title_groups INTEGER NOT NULL DEFAULT 0,
    edition_groups INTEGER NOT NULL DEFAULT 0,
    torrents INTEGER NOT NULL DEFAULT 0,
    forum_posts INTEGER NOT NULL DEFAULT 0,
    forum_threads INTEGER NOT NULL DEFAULT 0,
    title_group_comments INTEGER NOT NULL DEFAULT 0,
    request_comments INTEGER NOT NULL DEFAULT 0,
    artist_comments BIGINT NOT NULL DEFAULT 0,
    seeding INTEGER NOT NULL DEFAULT 0,
    leeching INTEGER NOT NULL DEFAULT 0,
    snatched INTEGER NOT NULL DEFAULT 0,
    seeding_size BIGINT NOT NULL DEFAULT 0,
    requests_filled BIGINT NOT NULL DEFAULT 0,
    collages_started BIGINT NOT NULL DEFAULT 0,
    requests_voted BIGINT NOT NULL DEFAULT 0,
    average_seeding_time BIGINT NOT NULL DEFAULT 0,
    invited BIGINT NOT NULL DEFAULT 0,
    invitations SMALLINT NOT NULL DEFAULT 0,
    bonus_points BIGINT NOT NULL DEFAULT 0,
    freeleech_tokens INT NOT NULL DEFAULT 0,
    passkey VARCHAR(32) NOT NULL,
    warned BOOLEAN NOT NULL DEFAULT FALSE,
    banned BOOLEAN NOT NULL DEFAULT FALSE,
    css_sheet_name VARCHAR(30) NOT NULL,
    current_streak INT NOT NULL DEFAULT 0,
    highest_streak INT NOT NULL DEFAULT 0,
    custom_title TEXT,
    max_snatches_per_day INT,
    irc_password VARCHAR(255),
    irc_site_embed_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    show_upload_method BOOLEAN NOT NULL DEFAULT FALSE,
    -- paranoia settings: information the user does not want other users to see
    paranoia_hidden_stats displayable_user_stats_enum[] NOT NULL DEFAULT '{}',
    paranoia_hidden_lists displayable_user_lists_enum[] NOT NULL DEFAULT '{}',

    UNIQUE(passkey)
);
INSERT INTO users (username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name)
VALUES ('creator', 'none@domain.com', 'none', '127.0.0.1', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'newbie', 'arcadia');
CREATE TABLE css_sheets (
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL REFERENCES users(id),
    name VARCHAR(30) UNIQUE NOT NULL,
    css TEXT NOT NULL,
    preview_image_url TEXT NOT NULL
);
INSERT INTO css_sheets (created_by_id, name, css, preview_image_url)
VALUES (1, 'arcadia', '', 'https://i.ibb.co/PvSfw9xz/Screenshot-2025-12-06-at-19-53-38-Home-Arcadia-Vault.png');
-- this needs to be done after the creation of css_sheets and users table
-- otherwise one of them isn't created yet
ALTER TABLE users
ADD CONSTRAINT fk_users_css_sheet
FOREIGN KEY (css_sheet_name)
REFERENCES css_sheets(name)
ON UPDATE CASCADE;
-- only logs forbidden actions from existing users, not from unauthenticated requests
CREATE TABLE unauthorized_accesses (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    user_id INT NOT NULL REFERENCES users(id),
    missing_permission user_permissions_enum NOT NULL,
    path TEXT NOT NULL
);

CREATE TYPE snatched_torrent_bonus_points_transferred_to_enum AS ENUM (
    'uploader',
    'current_seeders'
);
CREATE TYPE displayed_top_bar_stats_enum AS ENUM (
    'uploaded',
    'downloaded',
    'ratio',
    'torrents',
    'forum_posts',
    'seeding',
    'leeching',
    'seeding_size',
    'average_seeding_time',
    'bonus_points',
    'freeleech_tokens',
    'current_streak'
);
CREATE TYPE torrent_request_vote_currency_enum AS ENUM (
    'upload',
    'bonus_points'
);
CREATE TYPE available_shop_item_enum AS ENUM (
    'upload_amount',
    'freeleech_tokens',
    'user_class_promotion'
);
CREATE TABLE arcadia_settings (
    user_class_name_on_signup VARCHAR(30) NOT NULL REFERENCES user_classes(name) ON UPDATE CASCADE,
    default_css_sheet_name VARCHAR(30) NOT NULL REFERENCES css_sheets(name) ON UPDATE CASCADE,
    open_signups BOOLEAN NOT NULL,
    global_upload_factor SMALLINT NOT NULL,
    global_download_factor SMALLINT NOT NULL,
    global_snatch_bonus_points_cost_factor SMALLINT NOT NULL DEFAULT 100,
    global_snatch_bonus_points_reward_factor SMALLINT NOT NULL DEFAULT 100,
    logo_subtitle VARCHAR(60) DEFAULT NULL,
    approved_image_hosts TEXT[] NOT NULL DEFAULT '{}',
    upload_page_top_text TEXT,
    automated_message_on_signup TEXT,
    automated_message_on_signup_sender_id INT REFERENCES users(id),
    automated_message_on_signup_locked BOOLEAN,
    automated_message_on_signup_conversation_name VARCHAR(100),
    bonus_points_given_on_upload BIGINT NOT NULL DEFAULT 0,
    allow_uploader_set_torrent_bonus_points_cost BOOLEAN NOT NULL DEFAULT FALSE,
    default_torrent_bonus_points_cost BIGINT NOT NULL DEFAULT 0,
    shop_upload_base_price_per_gb BIGINT NOT NULL DEFAULT 100,
    shop_upload_discount_tiers JSONB NOT NULL DEFAULT '[{"threshold_gb": 10, "discount_percent": 10}, {"threshold_gb": 50, "discount_percent": 20}, {"threshold_gb": 100, "discount_percent": 30}]',
    shop_freeleech_token_base_price BIGINT NOT NULL DEFAULT 500,
    shop_freeleech_token_discount_tiers JSONB NOT NULL DEFAULT '[{"threshold": 5, "discount_percent": 10}, {"threshold": 10, "discount_percent": 15}, {"threshold": 25, "discount_percent": 25}]',
    bonus_points_alias VARCHAR(20) NOT NULL DEFAULT 'bonus points',
    torrent_max_release_date_allowed DATE DEFAULT NULL,
    torrent_bonus_points_cost_min BIGINT NOT NULL DEFAULT 0,
    torrent_bonus_points_cost_max BIGINT NOT NULL DEFAULT 0,
    snatched_torrent_bonus_points_transferred_to snatched_torrent_bonus_points_transferred_to_enum DEFAULT NULL,
    bonus_points_decimal_places SMALLINT NOT NULL DEFAULT 0,
    displayed_top_bar_stats displayed_top_bar_stats_enum[] NOT NULL DEFAULT '{uploaded,downloaded,bonus_points}',
    bonus_points_per_endpoint JSONB NOT NULL DEFAULT '[]',
    reward_bonus_points_per_seeding_client BOOLEAN NOT NULL DEFAULT FALSE,
    charge_bonus_points_on_resnatch BOOLEAN NOT NULL DEFAULT FALSE,
    displayable_user_stats displayable_user_stats_enum[] NOT NULL DEFAULT '{uploaded,real_uploaded,downloaded,real_downloaded,ratio,title_groups,edition_groups,torrents,forum_posts,forum_threads,title_group_comments,request_comments,artist_comments,seeding,leeching,snatched,seeding_size,requests_filled,collages_started,requests_voted,average_seeding_time,invited,invitations,bonus_points,freeleech_tokens,current_streak,highest_streak,joined_at,last_seen}',
    torrent_request_vote_currencies torrent_request_vote_currency_enum[] NOT NULL DEFAULT '{upload,bonus_points}',
    available_shop_items available_shop_item_enum[] NOT NULL DEFAULT '{upload_amount,freeleech_tokens,user_class_promotion}',
    default_user_uploaded_on_registration BIGINT NOT NULL DEFAULT 0,
    default_user_downloaded_on_registration BIGINT NOT NULL DEFAULT 1,
    default_user_bonus_points_on_registration BIGINT NOT NULL DEFAULT 0,
    default_user_freeleech_tokens_on_registration INT NOT NULL DEFAULT 0,
    display_image_host_drag_and_drop BOOLEAN NOT NULL DEFAULT FALSE,
    inactive_user_ban_after_days INT DEFAULT NULL,
    irc_webchat_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    irc_webchat_default_channels TEXT[] NOT NULL DEFAULT '{#general}',
    min_amount_tags_title_group INT NOT NULL DEFAULT 1,
    reseed_requestable_after_hours INT NOT NULL DEFAULT 72,
    custom_js_code TEXT DEFAULT NULL,
    custom_footer TEXT DEFAULT NULL,
    invitation_expiration_days INT NOT NULL DEFAULT 3,
    automated_message_on_torrent_marked_trumpable TEXT DEFAULT NULL
);
INSERT INTO arcadia_settings (user_class_name_on_signup, default_css_sheet_name, open_signups, global_upload_factor, global_download_factor, bonus_points_given_on_upload, allow_uploader_set_torrent_bonus_points_cost, default_torrent_bonus_points_cost, automated_message_on_signup, automated_message_on_signup_sender_id, automated_message_on_signup_locked, automated_message_on_signup_conversation_name, automated_message_on_torrent_marked_trumpable)
VALUES ('newbie', 'arcadia', TRUE, 100, 100, 100, FALSE, 0, 'Welcome to the site, {username}!', 1, FALSE, 'Welcome', 'You can fix your upload if you want to, but you are not obliged to.');
-- the groups of endpoints an API key is allowed to reach
CREATE TYPE api_key_scope_enum AS ENUM (
    'user',
    'torrents',
    'requests',
    'forum',
    'wiki'
);
CREATE TABLE api_keys (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    name VARCHAR(30) NOT NULL,
    -- the API key itself is never stored, only the sha256 hash of it
    value_hash BYTEA NOT NULL UNIQUE,
    -- the last four characters of the API key, to let the user recognize it
    last_four VARCHAR(4) NOT NULL,
    scopes api_key_scope_enum[] NOT NULL,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE
);
-- single use tokens allowing their bearer to set a new password without being authenticated.
-- a user only ever has one valid token, hence the unique constraint on user_id
CREATE TABLE password_reset_tokens (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    value VARCHAR(50) NOT NULL UNIQUE,
    user_id INT NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE
);
CREATE TYPE user_application_status_enum AS ENUM (
    'pending',
    'accepted',
    'rejected'
);
CREATE TABLE user_applications (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    body TEXT NOT NULL,
    referral TEXT NOT NULL,
    email TEXT NOT NULL,
    applied_from_ip INET NOT NULL,
    staff_note TEXT NOT NULL DEFAULT '',
    status user_application_status_enum NOT NULL DEFAULT 'pending'
);
CREATE TABLE invitations (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    invitation_key VARCHAR(50) NOT NULL,
    message TEXT NOT NULL,
    inviter_notes TEXT,
    sender_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    receiver_email VARCHAR(255) NOT NULL,
    user_application_id BIGINT REFERENCES user_applications(id) ON DELETE SET NULL,
    receiver_id INT REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE user_warnings (
    id BIGSERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    reason TEXT NOT NULL,
    ban boolean NOT NULL,
    created_by_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    removed_at TIMESTAMP WITH TIME ZONE,
    removed_by_id INT REFERENCES users(id) ON DELETE SET NULL
);
CREATE TABLE user_staff_notes (
    id BIGSERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    content TEXT NOT NULL,
    created_by_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE
);
CREATE TABLE gifts (
    id BIGSERIAL PRIMARY KEY,
    sent_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    message TEXT NOT NULL,
    sender_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    receiver_id INT NOT NULL REFERENCES users(id) ON DELETE SET NULL,
    bonus_points BIGINT NOT NULL DEFAULT 0,
    freeleech_tokens INT NOT NULL DEFAULT 0
);
CREATE TABLE artists (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    aliases VARCHAR(255) [] NOT NULL DEFAULT '{}',
    description TEXT NOT NULL,
    pictures TEXT [] NOT NULL,
    external_links TEXT [] NOT NULL DEFAULT '{}',
    created_by_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    title_groups_amount INT NOT NULL DEFAULT 0,
    edition_groups_amount INT NOT NULL DEFAULT 0,
    torrents_amount INT NOT NULL DEFAULT 0,
    seeders_amount INT NOT NULL DEFAULT 0,
    leechers_amount INT NOT NULL DEFAULT 0,
    snatches_amount INT NOT NULL DEFAULT 0,
    total_size BIGINT NOT NULL DEFAULT 0,
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE TABLE similar_artists (
    artist_1_id BIGINT NOT NULL,
    artist_2_id BIGINT NOT NULL,
    PRIMARY KEY (artist_1_id, artist_2_id),
    FOREIGN KEY (artist_1_id) REFERENCES artists(id) ON DELETE CASCADE,
    FOREIGN KEY (artist_2_id) REFERENCES artists(id) ON DELETE CASCADE
);
CREATE TABLE master_groups (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255),
    -- name_aliases VARCHAR(255)[],
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    -- description TEXT NOT NULL,
    -- original_language VARCHAR(50) NOT NULL,
    -- country_from VARCHAR(50) NOT NULL,
    -- tags VARCHAR(50)[] NOT NULL,
    -- category VARCHAR(25), -- should only be used for TV-Shows (scripted, reality-tv, etc.)
    -- covers TEXT[],
    -- banners TEXT[],
    -- fan_arts TEXT[],
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE
    SET NULL
);
CREATE TABLE similar_master_groups (
    group_1_id INT NOT NULL,
    group_2_id INT NOT NULL,
    PRIMARY KEY (group_1_id, group_2_id),
    FOREIGN KEY (group_1_id) REFERENCES master_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (group_2_id) REFERENCES master_groups(id) ON DELETE CASCADE
);
CREATE TABLE series (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    tags TEXT [] NOT NULL,
    covers TEXT [] NOT NULL,
    banners TEXT [] NOT NULL,
    created_by_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE TYPE content_type_enum AS ENUM (
    'movie',
    'video',
    'tv_show',
    'music',
    'podcast',
    'software',
    'book',
    'live_performance',
    'collection'
);
CREATE TYPE title_group_category_enum AS ENUM (
    'Ep',
    'Album',
    'Single',
    'Soundtrack',
    'Anthology',
    'Compilation',
    'Remix',
    'Bootleg',
    'Mixtape',
    'ConcertRecording',
    'DjMix',
    'FeatureFilm',
    'ShortFilm',
    'Game',
    'Program',
    'Illustrated',
    'Periodical',
    'Book',
    'Article',
    'Manual',
    'Other'
);
CREATE TYPE platform_enum AS ENUM(
    'Linux',
    'MacOS',
    'IOS',
    'AppleBandaiPippin',
    'AppleII',
    'Android',
    'Windows',
    'DOS',
    'Xbox',
    'Xbox360',
    'XboxOne',
    'XboxSeriesXS',
    'Playstation',
    'Playstation2',
    'Playstation3',
    'Playstation4',
    'Playstation5',
    'PlaystationPortable',
    'PlaystationVita',
    'GameBoy',
    'GameBoyAdvance',
    'GameBoyColor',
    'NES',
    'SNES',
    'N64',
    'ThreeDS',
    'New3DS',
    'DS',
    'VirtualBoy',
    'GameCube',
    'Wii',
    'WiiU',
    'Switch',
    'Switch2',
    'Dreamcast',
    'GameGear',
    'MasterSystem',
    'MegaDrive',
    'Pico',
    'Saturn',
    'SG1000',
    'Atari2600',
    'Atari5200',
    'Atari7800',
    'AtariJaguar',
    'AtariLynx',
    'AtariST',
    'AmstradCPC',
    'BandaiWonderSwan',
    'BandaiWonderSwanColor',
    'Commodore64',
    'Commodore128',
    'CommodoreAmiga',
    'AmigaCD32',
    'CommodorePlus4',
    'CommodoreVIC20'
);
CREATE TYPE language_enum AS ENUM(
   'Albanian',
   'Arabic',
   'Belarusian',
   'Bengali',
   'Bosnian',
   'Bulgarian',
   'Cantonese',
   'Catalan',
   'Chinese',
   'Chinese Simplified',
   'Chinese Traditional',
   'Croatian',
   'Czech',
   'Danish',
   'Dutch',
   'English',
   'Estonian',
   'Finnish',
   'French',
   'German',
   'Greek',
   'Hebrew',
   'Hindi',
   'Hungarian',
   'Icelandic',
   'Indonesian',
   'Italian',
   'Japanese',
   'Kannada',
   'Korean',
   'Latvian',
   'Lithuanian',
   'Macedonian',
   'Malay',
   'Malayalam',
   'Mandarin',
   'Nepali',
   'Norwegian',
   'Persian',
   'Polish',
   'Portuguese',
   'Romanian',
   'Russian',
   'Serbian',
   'Slovak',
   'Slovenian',
   'Spanish',
   'Swedish',
   'Tamil',
   'Tagalog',
   'Telugu',
   'Thai',
   'Turkish',
   'Ukrainian',
   'Vietnamese',
   'Wolof',
   'Other'
);
CREATE TABLE title_groups (
    id SERIAL PRIMARY KEY,
    master_group_id INT,
    name TEXT NOT NULL,
    name_aliases TEXT [],
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    description TEXT NOT NULL,
    platform platform_enum,
    original_language language_enum,
    original_release_date DATE,
    original_release_date_only_year_known BOOLEAN NOT NULL DEFAULT FALSE,
    tagline TEXT,
    country_from TEXT,
    covers TEXT [] NOT NULL,
    external_links TEXT [] NOT NULL,
    trailers TEXT [] NOT NULL,
    category title_group_category_enum,
    content_type content_type_enum NOT NULL,
    public_ratings JSONB NOT NULL,
    screenshots TEXT[] NOT NULL,
    series_id BIGINT,
    FOREIGN KEY (master_group_id) REFERENCES master_groups(id) ON DELETE
    SET NULL,
        FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE
    SET NULL,
        FOREIGN KEY (series_id) REFERENCES series(id) ON DELETE
    SET NULL
);
-- lets the catalog stats of a series count its title groups without scanning the whole table
CREATE INDEX title_groups_series_id_index ON title_groups (series_id);
CREATE TABLE similar_title_groups (
    group_1_id INT NOT NULL,
    group_2_id INT NOT NULL,
    note TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    PRIMARY KEY (group_1_id, group_2_id),
    -- enforce canonical ordering so each bidirectional link is stored only once
    -- (as the (min, max) pair), letting the primary key deduplicate links
    -- regardless of the order the IDs were passed in
    CHECK (group_1_id < group_2_id),
    FOREIGN KEY (group_1_id) REFERENCES title_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (group_2_id) REFERENCES title_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by_id) REFERENCES users(id)
);
CREATE TABLE title_group_tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR(40) NOT NULL,
    synonyms VARCHAR(40)[] DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    deleted_by_id INT DEFAULT NULL,
    deletion_reason VARCHAR(255) DEFAULT NULL,
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (deleted_by_id) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE (name)
);

CREATE OR REPLACE FUNCTION enforce_unique_title_group_tag_synonyms()
RETURNS TRIGGER AS $$
DECLARE
    existing VARCHAR(40);
    conflict_tag_name VARCHAR(40);
BEGIN
    -- Loop through each synonym in the new row
    FOREACH existing IN ARRAY NEW.synonyms LOOP
        -- Check if this synonym exists in any other non-deleted row (or if it's an existing tag name)
        SELECT name INTO conflict_tag_name
        FROM title_group_tags
        WHERE id <> NEW.id
          AND deleted_at IS NULL
          AND (existing = ANY(synonyms) OR existing = name)
        LIMIT 1;

        IF conflict_tag_name IS NOT NULL THEN
            RAISE EXCEPTION 'Synonym "%" already exists in title_group_tag "%" ', existing, conflict_tag_name;
        END IF;
    END LOOP;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_unique_synonyms
BEFORE INSERT OR UPDATE ON title_group_tags
FOR EACH ROW
EXECUTE FUNCTION enforce_unique_title_group_tag_synonyms();

CREATE TABLE title_group_applied_tags (
    title_group_id INT NOT NULL,
    tag_id INT NOT NULL,
    created_by_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (title_group_id, tag_id),
    FOREIGN KEY (title_group_id) REFERENCES title_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES title_group_tags(id) ON DELETE CASCADE
);
CREATE TYPE artist_role_enum AS ENUM (
    'main',
    'guest',
    'producer',
    'director',
    'cinematographer',
    'actor',
    'writer',
    'composer',
    'remixer',
    'conductor',
    'dj_compiler',
    'arranger',
    'host',
    'author',
    'illustrator',
    'editor',
    'developer',
    'designer',
    'creator',
    'performer',
    'presenter',
    'contributor'
);
CREATE TABLE affiliated_artists (
    id BIGSERIAL PRIMARY KEY,
    title_group_id INT NOT NULL,
    artist_id BIGINT NOT NULL,
    roles artist_role_enum[] NOT NULL,
    nickname VARCHAR(255),
    created_by_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    FOREIGN KEY (title_group_id) REFERENCES title_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE
    SET NULL,
    UNIQUE(title_group_id, artist_id)
);
-- lets the catalog stats of an artist find its title groups without scanning the whole table
CREATE INDEX affiliated_artists_artist_id_index ON affiliated_artists (artist_id);
-- for web: if it is a DL or a RIP should be specified at the torrent level
CREATE TYPE source_enum AS ENUM (
    'CD',
    'Vinyl',
    'Web',
    'Soundboard',
    'SACD',
    'DAT',
    'Cassette',
    'Blu-Ray',
    'LaserDisc',
    'DVD',
    'HD-DVD',
    'HDTV',
    'PDTV',
    'TV',
    'VHS',
    'Mixed',
    'Physical Book',
    'Unknown'
);
CREATE TABLE edition_groups (
    id SERIAL PRIMARY KEY,
    title_group_id INT NOT NULL,
    name TEXT,
    release_date DATE,
    release_date_only_year_known BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    description TEXT,
    distributor VARCHAR(255),
    covers TEXT [] NOT NULL,
    external_links TEXT [] NOT NULL,
    source source_enum,
    additional_information JSONB,
    FOREIGN KEY (title_group_id) REFERENCES title_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE
    SET NULL
);
CREATE TYPE audio_codec_enum AS ENUM (
    'mp2',
    'mp3',
    'aac',
    'ac3',
    'dts',
    'flac',
    'pcm',
    'true-hd',
    'opus',
    'dsd',
    'cook',
    'wma'
);

CREATE TYPE video_resolution_enum AS ENUM (
    'Other',
    'NTSC',
    'PAL',
    '360p',
    '480p',
    '480i',
    '576p',
    '576i',
    '720p',
    '1080p',
    '1080i',
    '1440p',
    '2160p',
    '4320p'
);

CREATE TYPE audio_bitrate_sampling_enum AS ENUM(
    '64',
    '128',
    '192',
    '256',
    '320',
    'APS (VBR)',
    'V2 (VBR)',
    'V1 (VBR)',
    'APX (VBR)',
    'V0 (VBR)',
    'Lossless',
    '24bit Lossless',
    'DSD64',
    'DSD128',
    'DSD256',
    'DSD512',
    'other'
);
CREATE TYPE audio_channels_enum AS ENUM (
    '1.0',
    '2.0',
    '2.1',
    '5.0',
    '5.1',
    '7.1'
);
CREATE TYPE video_codec_enum AS ENUM(
    'mpeg1',
    'mpeg2',
    'XviD',
    'DivX',
    'h264',
    'h265',
    'vc-1',
    'vp9',
    'av1',
    'BD50',
    'UHD100',
    'DVD5',
    'DVD9',
    'VP6',
    'RV40',
    'WMV2',
    'WMV1'
);
CREATE TYPE features_enum AS ENUM('HDR', 'HDR 10', 'HDR 10+', 'DV', 'Commentary', 'Remux', '3D', 'Cue', 'OCR');
CREATE TYPE extras_enum AS ENUM('booklet', 'manual', 'behind_the_scenes', 'deleted_scenes', 'featurette', 'trailer', 'other');
CREATE TABLE torrents (
    id SERIAL PRIMARY KEY,
    upload_factor SMALLINT NOT NULL DEFAULT 100,
    download_factor SMALLINT NOT NULL DEFAULT 100,
    seeders BIGINT NOT NULL DEFAULT 0,
    leechers BIGINT NOT NULL DEFAULT 0,
    times_completed INT NOT NULL DEFAULT 0,
    grabbed BIGINT NOT NULL DEFAULT 0,
    edition_group_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    deleted_by_id INT DEFAULT NULL,
    info_hash BYTEA NOT NULL CHECK(octet_length(info_hash) = 20),
    info_dict BYTEA NOT NULL,
    languages language_enum[] NOT NULL DEFAULT ARRAY[]::language_enum[],
    release_name TEXT NOT NULL,
    -- maybe change the size
    release_group VARCHAR(30),
    description TEXT,
    file_amount_per_type JSONB NOT NULL,
    uploaded_as_anonymous BOOLEAN NOT NULL DEFAULT FALSE,
    upload_method VARCHAR(50) NOT NULL DEFAULT 'manual',
    file_list JSONB NOT NULL,
    -- maybe change the size to the max length of a file name in a torrent
    mediainfo TEXT,
    trumpable TEXT,
    staff_checked BOOLEAN NOT NULL DEFAULT FALSE,
    container VARCHAR(8) NOT NULL,
    -- in bytes
    size BIGINT NOT NULL,

    -- audio
    duration INT,
    -- in seconds
    audio_codec audio_codec_enum,
    audio_bitrate INT,
    -- in kb/s, taken from mediainfo
    audio_bitrate_sampling audio_bitrate_sampling_enum,
    audio_channels audio_channels_enum,
    -- audio

    -- video
    video_codec video_codec_enum,
    features features_enum [] NOT NULL DEFAULT ARRAY[]::features_enum[],
    subtitle_languages language_enum[] NOT NULL DEFAULT ARRAY[]::language_enum[],
    video_resolution video_resolution_enum,
    video_resolution_other_x INT,
    video_resolution_other_y INT,

    extras extras_enum[] DEFAULT ARRAY[]::extras_enum[],
    extra_text VARCHAR(50),

    bonus_points_snatch_cost BIGINT NOT NULL DEFAULT 0,

    FOREIGN KEY (edition_group_id) REFERENCES edition_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE (info_hash)
);
CREATE TABLE title_group_comments (
    id BIGSERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    title_group_id INT NOT NULL,
    locked BOOLEAN NOT NULL DEFAULT FALSE,
    refers_to_torrent_id INT,
    answers_to_comment_id BIGINT,
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (title_group_id) REFERENCES title_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (refers_to_torrent_id) REFERENCES torrents(id) ON DELETE SET NULL,
    FOREIGN KEY (answers_to_comment_id) REFERENCES title_group_comments(id) ON DELETE SET NULL
);
CREATE TABLE torrent_requests (
    id BIGSERIAL PRIMARY KEY,
    title_group_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    filled_by_user_id INT,
    filled_by_torrent_id INT,
    filled_at TIMESTAMP WITH TIME ZONE,
    edition_name TEXT,
    source source_enum[] NOT NULL DEFAULT ARRAY[]::source_enum[],
    release_group VARCHAR(20),
    description TEXT,
    languages language_enum[] NOT NULL DEFAULT ARRAY[]::language_enum[],
    container VARCHAR(8)[] NOT NULL DEFAULT ARRAY[]::VARCHAR(8)[],
    -- Audio
    audio_codec audio_codec_enum[] NOT NULL DEFAULT ARRAY[]::audio_codec_enum[],
    audio_channels audio_channels_enum[] NOT NULL DEFAULT ARRAY[]::audio_channels_enum[],
    audio_bitrate_sampling audio_bitrate_sampling_enum[] NOT NULL DEFAULT ARRAY[]::audio_bitrate_sampling_enum[],
    -- Video
    video_codec video_codec_enum[] NOT NULL DEFAULT ARRAY[]::video_codec_enum[],
    features features_enum[] NOT NULL DEFAULT ARRAY[]::features_enum[],
    subtitle_languages language_enum[] NOT NULL DEFAULT ARRAY[]::language_enum[],
    video_resolution video_resolution_enum[] NOT NULL DEFAULT ARRAY[]::video_resolution_enum[],
    video_resolution_other_x INT,
    video_resolution_other_y INT,
    FOREIGN KEY (title_group_id) REFERENCES title_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by_id) REFERENCES users(id),
    FOREIGN KEY (filled_by_user_id) REFERENCES users(id),
    FOREIGN KEY (filled_by_torrent_id) REFERENCES torrents(id)
);
CREATE TABLE torrent_request_votes(
    id BIGSERIAL PRIMARY KEY,
    torrent_request_id BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    bounty_upload BIGINT NOT NULL DEFAULT 0,
    bounty_bonus_points BIGINT NOT NULL DEFAULT 0,
    FOREIGN KEY (torrent_request_id) REFERENCES torrent_requests(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE TABLE torrent_request_comments (
    id BIGSERIAL PRIMARY KEY,
    torrent_request_id BIGINT NOT NULL REFERENCES torrent_requests(id) ON DELETE CASCADE,
    created_by_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
CREATE TABLE torrent_reports (
    id BIGSERIAL PRIMARY KEY,
    reported_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    reported_by_id INT NOT NULL,
    description TEXT NOT NULL,
    reported_torrent_id INT NOT NULL,
    FOREIGN KEY (reported_by_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (reported_torrent_id) REFERENCES torrents(id) ON DELETE CASCADE
);
CREATE TABLE peers (
    peer_id bytea NOT NULL,
    ip INET NOT NULL,
    port INT NOT NULL,
    agent varchar(64) NOT NULL,
    uploaded bigint NOT NULL,
    downloaded bigint NOT NULL,
    "left" bigint NOT NULL,
    seeder boolean NOT NULL,
    created_at timestamp without time zone DEFAULT NULL,
    updated_at timestamp without time zone DEFAULT NULL,
    torrent_id integer NOT NULL,
    user_id integer NOT NULL,
    -- connectable boolean NOT NULL DEFAULT FALSE,
    active boolean NOT NULL,
    -- visible boolean NOT NULL,
    PRIMARY KEY (user_id, torrent_id, peer_id)
);
CREATE INDEX peers_idx_seeder_user_id ON peers (seeder, user_id);
CREATE INDEX peers_torrent_id_foreign ON peers (torrent_id);
CREATE INDEX peers_active_index ON peers (active);
ALTER TABLE peers
ADD CONSTRAINT peers_torrent_id_foreign FOREIGN KEY (torrent_id) REFERENCES torrents (id) ON DELETE CASCADE ON UPDATE CASCADE;
ALTER TABLE peers
ADD CONSTRAINT peers_user_id_foreign FOREIGN KEY (user_id) REFERENCES users (id) ON UPDATE CASCADE;
CREATE TABLE torrent_activities (
    id BIGSERIAL PRIMARY KEY,
    torrent_id INT NOT NULL,
    user_id INT NOT NULL,
    -- .torrent file downloaded
    grabbed_at TIMESTAMP WITH TIME ZONE,
    -- completed event from torrent client
    completed_at TIMESTAMP WITH TIME ZONE,
    first_seen_seeding_at TIMESTAMP WITH TIME ZONE,
    last_seen_seeding_at TIMESTAMP WITH TIME ZONE,
    total_seed_time BIGINT NOT NULL DEFAULT 0,
    bonus_points BIGINT NOT NULL DEFAULT 0,
    uploaded BIGINT NOT NULL DEFAULT 0,
    real_uploaded BIGINT NOT NULL DEFAULT 0,
    downloaded BIGINT NOT NULL DEFAULT 0,
    real_downloaded BIGINT NOT NULL DEFAULT 0,

    FOREIGN KEY (torrent_id) REFERENCES torrents(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id),

    UNIQUE (torrent_id, user_id)
);
CREATE TABLE entities (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    pictures TEXT[] NOT NULL,
    created_by_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    title_groups_amount INT NOT NULL DEFAULT 0,
    edition_groups_amount INT NOT NULL DEFAULT 0,
    torrents_amount INT NOT NULL DEFAULT 0,
    seeders_amount INT NOT NULL DEFAULT 0,
    leechers_amount INT NOT NULL DEFAULT 0,
    snatches_amount INT NOT NULL DEFAULT 0,
    FOREIGN KEY (created_by_id) REFERENCES users(id)
);
CREATE TYPE entity_role_enum AS ENUM (
    'producer',
    'developer',
    'designer',
    'label'
);
CREATE TABLE affiliated_entities (
    id BIGSERIAL PRIMARY KEY,
    title_group_id INT NOT NULL,
    entity_id BIGINT NOT NULL,
    created_by_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    roles entity_role_enum[] NOT NULL,
    FOREIGN KEY (title_group_id) REFERENCES title_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE SET NULL
);
CREATE TYPE collage_category_enum AS ENUM (
    'Personal',
    'Staff Picks',
    'External',
    'Theme'
);
CREATE TABLE collage (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    name VARCHAR NOT NULL,
    cover TEXT,
    description TEXT NOT NULL,
    tags VARCHAR[] NOT NULL,
    category collage_category_enum NOT NULL,
    FOREIGN KEY (created_by_id) REFERENCES users(id)
);
CREATE TABLE collage_entry (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL REFERENCES users(id),
    collage_id BIGINT NOT NULL REFERENCES collage(id),
    title_group_id INT NOT NULL REFERENCES title_groups(id),
    note TEXT
);
-- prevent duplicate entries in a collage
CREATE UNIQUE INDEX unique_title_group_per_collage
ON collage_entry (collage_id, title_group_id)
WHERE title_group_id IS NOT NULL;
CREATE TABLE forum_categories (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,

    FOREIGN KEY (created_by_id) REFERENCES users(id)
);
INSERT INTO forum_categories (created_by_id, name, sort_order) VALUES (1, 'Site', 1);
CREATE TYPE forum_thread_sort_by_enum AS ENUM (
    'latest_post',
    'created_at',
    'name',
    'posts_amount',
    'views_count'
);
CREATE TYPE forum_thread_sort_direction_enum AS ENUM (
    'ascending',
    'descending'
);
CREATE TABLE forum_sub_categories (
    id SERIAL PRIMARY KEY NOT NULL,
    forum_category_id INT NOT NULL,
    name TEXT NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    threads_amount BIGINT NOT NULL DEFAULT 0,
    posts_amount BIGINT NOT NULL DEFAULT 0,
    forbidden_classes VARCHAR(50) [] NOT NULL DEFAULT ARRAY[]::VARCHAR(50)[],
    new_threads_restricted BOOLEAN NOT NULL DEFAULT FALSE,
    thread_sort_by forum_thread_sort_by_enum NOT NULL DEFAULT 'latest_post',
    thread_sort_direction forum_thread_sort_direction_enum NOT NULL DEFAULT 'descending',

    FOREIGN KEY (created_by_id) REFERENCES users(id),
    FOREIGN KEY (forum_category_id) REFERENCES forum_categories(id)
);
INSERT INTO forum_sub_categories (created_by_id, forum_category_id, name, sort_order, threads_amount, posts_amount) VALUES (1, 1, 'Announcements', 1, 1, 1);
CREATE TABLE forum_sub_category_allowed_posters (
    forum_sub_category_id INT NOT NULL,
    user_id INT NOT NULL,

    PRIMARY KEY (forum_sub_category_id, user_id),
    FOREIGN KEY (forum_sub_category_id) REFERENCES forum_sub_categories(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE TABLE forum_threads (
    id BIGSERIAL PRIMARY KEY,
    forum_sub_category_id INT NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    posts_amount BIGINT NOT NULL DEFAULT 0,
    pinned BOOLEAN NOT NULL DEFAULT FALSE,
    locked BOOLEAN NOT NULL DEFAULT FALSE,
    views_count BIGINT NOT NULL DEFAULT 0,

    FOREIGN KEY (created_by_id) REFERENCES users(id),
    FOREIGN KEY (forum_sub_category_id) REFERENCES forum_sub_categories(id)
);
INSERT INTO forum_threads (created_by_id, forum_sub_category_id, name, posts_amount) VALUES (1, 1, 'Welcome to the site!', 1);
CREATE TABLE forum_posts (
    id BIGSERIAL PRIMARY KEY,
    forum_thread_id BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    content TEXT NOT NULL,
    sticky BOOLEAN NOT NULL DEFAULT FALSE,
    locked BOOLEAN NOT NULL DEFAULT FALSE,

    FOREIGN KEY (created_by_id) REFERENCES users(id),
    FOREIGN KEY (forum_thread_id) REFERENCES forum_threads(id)
);
INSERT INTO forum_posts (created_by_id, forum_thread_id, content) VALUES (1, 1, 'Welcome!');
CREATE TABLE forum_thread_reads (
    user_id INT NOT NULL,
    forum_thread_id BIGINT NOT NULL,
    last_read_post_id BIGINT NOT NULL,
    read_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, forum_thread_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (forum_thread_id) REFERENCES forum_threads(id) ON DELETE CASCADE,
    FOREIGN KEY (last_read_post_id) REFERENCES forum_posts(id) ON DELETE CASCADE
);
CREATE TABLE forum_polls (
    id BIGSERIAL PRIMARY KEY,
    forum_thread_id BIGINT NOT NULL UNIQUE,
    question TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,

    FOREIGN KEY (forum_thread_id) REFERENCES forum_threads(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by_id) REFERENCES users(id)
);
CREATE TABLE forum_poll_options (
    id BIGSERIAL PRIMARY KEY,
    forum_poll_id BIGINT NOT NULL,
    content TEXT NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,

    FOREIGN KEY (forum_poll_id) REFERENCES forum_polls(id) ON DELETE CASCADE
);
CREATE TABLE forum_poll_votes (
    forum_poll_id BIGINT NOT NULL,
    forum_poll_option_id BIGINT,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    PRIMARY KEY (forum_poll_id, user_id),
    FOREIGN KEY (forum_poll_id) REFERENCES forum_polls(id) ON DELETE CASCADE,
    FOREIGN KEY (forum_poll_option_id) REFERENCES forum_poll_options(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
-- emojis usable as reactions. an emoji is either a unicode character or a small image
-- stored in the database, never both
CREATE TABLE emojis (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    unicode_character TEXT,
    image BYTEA,
    image_mime_type TEXT,
    sort_order SMALLINT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    CHECK ((unicode_character IS NULL) != (image IS NULL)),
    CHECK ((image IS NULL) = (image_mime_type IS NULL))
);

CREATE TABLE forum_post_reactions (
    forum_post_id BIGINT NOT NULL REFERENCES forum_posts(id) ON DELETE CASCADE,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emoji_id INT NOT NULL REFERENCES emojis(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    PRIMARY KEY (forum_post_id, user_id, emoji_id)
);
CREATE INDEX forum_post_reactions_emoji_id_idx ON forum_post_reactions (emoji_id);
CREATE TABLE wiki_articles (
    id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_by_id INT NOT NULL,
    body TEXT NOT NULL,

    FOREIGN KEY (created_by_id) REFERENCES users(id)
);
CREATE TABLE similar_wiki_articles (
    wiki_article_id_1 BIGINT NOT NULL,
    wiki_article_id_2 BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,

    PRIMARY KEY (wiki_article_id_1, wiki_article_id_2),
    -- enforce canonical ordering so each bidirectional link is stored only once
    -- (as the (min, max) pair), letting the primary key deduplicate links
    -- regardless of the order the IDs were passed in
    CHECK (wiki_article_id_1 < wiki_article_id_2),
    FOREIGN KEY (wiki_article_id_1) REFERENCES wiki_articles(id) ON DELETE CASCADE,
    FOREIGN KEY (wiki_article_id_2) REFERENCES wiki_articles(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by_id) REFERENCES users(id)
);
CREATE INDEX idx_similar_wiki_articles_id_2 ON similar_wiki_articles(wiki_article_id_2);
CREATE TABLE title_group_bookmarks (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    user_id INT NOT NULL,
    title_group_id INT NOT NULL,
    description TEXT,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (title_group_id) REFERENCES title_groups(id) ON DELETE CASCADE
);
CREATE TABLE conversations (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    subject VARCHAR(255) NOT NULL,
    sender_id INT NOT NULL,
    receiver_id INT NOT NULL,
    sender_last_seen_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    receiver_last_seen_at TIMESTAMP WITH TIME ZONE,
    locked BOOLEAN NOT NULL DEFAULT FALSE,

    FOREIGN KEY (sender_id) REFERENCES users(id),
    FOREIGN KEY (receiver_id) REFERENCES users(id)
);
CREATE TABLE conversation_messages (
    id BIGSERIAL PRIMARY KEY,
    conversation_id BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    created_by_id INT NOT NULL,
    content TEXT NOT NULL,

    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (created_by_id) REFERENCES users(id)
);
CREATE TABLE staff_pms (
	id BIGSERIAL PRIMARY KEY,
	created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
	subject TEXT NOT NULL,
	created_by_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	resolved BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE TABLE staff_pm_messages (
	id BIGSERIAL PRIMARY KEY,
	staff_pm_id BIGINT NOT NULL REFERENCES staff_pms(id) ON DELETE CASCADE,
	created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
	created_by_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	content TEXT NOT NULL
);
CREATE TABLE subscriptions_collages (
    id BIGSERIAL PRIMARY KEY,
    collage_id BIGINT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    FOREIGN KEY (collage_id) REFERENCES collage(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE (collage_id, user_id)
);
CREATE TABLE notifications_collages (
    id BIGSERIAL PRIMARY KEY,
    collage_id BIGINT NOT NULL,
    title_group_id INT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    read_status BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (collage_id) REFERENCES collage(id) ON DELETE CASCADE,
    FOREIGN KEY (title_group_id) REFERENCES title_groups(id) ON DELETE CASCADE
);
-- notifies of new threads within a forum sub-category
CREATE TABLE subscriptions_forum_sub_category_threads (
    id BIGSERIAL PRIMARY KEY,
    forum_sub_category_id INT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    FOREIGN KEY (forum_sub_category_id) REFERENCES forum_sub_categories(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE (forum_sub_category_id, user_id)
);
CREATE TABLE notifications_forum_sub_category_threads (
    id BIGSERIAL PRIMARY KEY,
    forum_sub_category_id INT NOT NULL,
    forum_thread_id BIGINT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    read_status BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (forum_thread_id) REFERENCES forum_threads(id) ON DELETE CASCADE
);
-- notifies of new posts within a forum thread
CREATE TABLE subscriptions_forum_thread_posts (
    id BIGSERIAL PRIMARY KEY,
    forum_thread_id BIGINT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    FOREIGN KEY (forum_thread_id) REFERENCES forum_threads(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE (forum_thread_id, user_id)
);
CREATE TABLE notifications_forum_thread_posts (
    id BIGSERIAL PRIMARY KEY,
    forum_thread_id BIGINT NOT NULL,
    forum_post_id BIGINT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    read_status BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (forum_post_id) REFERENCES forum_posts(id) ON DELETE CASCADE
);
-- notifies of new torrents within a title group
CREATE TABLE subscriptions_title_group_torrents (
    id BIGSERIAL PRIMARY KEY,
    title_group_id INT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    FOREIGN KEY (title_group_id) REFERENCES title_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE (title_group_id, user_id)
);
CREATE TABLE notifications_title_group_torrents  (
    id BIGSERIAL PRIMARY KEY,
    torrent_id INT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    read_status BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (torrent_id) REFERENCES torrents(id) ON DELETE CASCADE
);
-- notifies of new comments within a title group
CREATE TABLE subscriptions_title_group_comments (
    id BIGSERIAL PRIMARY KEY,
    title_group_id INT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    FOREIGN KEY (title_group_id) REFERENCES title_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE (title_group_id, user_id)
);
CREATE TABLE notifications_title_group_comments (
    id BIGSERIAL PRIMARY KEY,
    title_group_id INT NOT NULL,
    title_group_comment_id BIGINT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    read_status BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (title_group_comment_id) REFERENCES title_group_comments(id) ON DELETE CASCADE
);
-- notifies of new comments on a torrent request
CREATE TABLE subscriptions_torrent_request_comments (
    id BIGSERIAL PRIMARY KEY,
    torrent_request_id BIGINT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    FOREIGN KEY (torrent_request_id) REFERENCES torrent_requests(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE (torrent_request_id, user_id)
);
CREATE TABLE notifications_torrent_request_comments (
    id BIGSERIAL PRIMARY KEY,
    torrent_request_id BIGINT NOT NULL,
    torrent_request_comment_id BIGINT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    read_status BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (torrent_request_comment_id) REFERENCES torrent_request_comments(id) ON DELETE CASCADE
);
-- notifies of title groups newly affiliated to an artist
CREATE TABLE subscriptions_artist_title_groups (
    id BIGSERIAL PRIMARY KEY,
    artist_id BIGINT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE (artist_id, user_id)
);
CREATE TABLE notifications_artist_title_groups (
    id BIGSERIAL PRIMARY KEY,
    artist_id BIGINT NOT NULL,
    title_group_id INT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    read_status BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE,
    FOREIGN KEY (title_group_id) REFERENCES title_groups(id) ON DELETE CASCADE
);
CREATE TABLE notifications_staff_pm_messages (
    id BIGSERIAL PRIMARY KEY,
    staff_pm_id BIGINT NOT NULL,
    staff_pm_message_id BIGINT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    read_status BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (staff_pm_message_id) REFERENCES staff_pm_messages(id) ON DELETE CASCADE
);
CREATE TABLE notifications_reseed_requests (
    torrent_id      INT NOT NULL REFERENCES torrents(id) ON DELETE CASCADE,
    user_id         INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    requested_by_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    read_status     BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (torrent_id, user_id, requested_by_id)
);
CREATE INDEX idx_notifications_reseed_requests_user ON notifications_reseed_requests(user_id, read_status);
CREATE TYPE torrent_deletion_reason_enum AS ENUM (
    'trumped',
    'duplicate',
    'other'
);
-- one row per deleted torrent. torrent_id is intentionally not a foreign key so
-- the historical record survives a hard delete of the torrent row.
CREATE TABLE torrent_deletions (
    torrent_id INT PRIMARY KEY,
    title_group_name TEXT NOT NULL,
    deletion_reason torrent_deletion_reason_enum NOT NULL,
    extra_information TEXT,
    replacement_torrent_id INT,
    deleted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_by_id INT,
    FOREIGN KEY (replacement_torrent_id) REFERENCES torrents(id) ON DELETE SET NULL,
    FOREIGN KEY (deleted_by_id) REFERENCES users(id) ON DELETE SET NULL
);
-- per-user notification rows for torrent deletions
CREATE TABLE torrent_deletion_notifications (
    user_id INT NOT NULL,
    torrent_id INT NOT NULL,
    read_status BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (user_id, torrent_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (torrent_id) REFERENCES torrent_deletions(torrent_id) ON DELETE CASCADE
);
CREATE TYPE announce_error_code_enum AS ENUM (
    'torrent_client_not_in_whitelist',
    'info_hash_not_found',
    'torrent_is_deleted',
    'peers_per_torrent_per_user_limit',
    'snatch_limit_reached',
    'insufficient_bonus_points'
);
-- errors returned by the tracker to a user's announces, written by the tracker and
-- removed by a periodic task once resolved (successful announce from the same peer) or stale
CREATE TABLE announce_errors (
    user_id INT NOT NULL,
    info_hash BYTEA NOT NULL,
    -- NULL when the info_hash is not known by the tracker
    torrent_id INT,
    -- last error returned by the tracker
    error_code announce_error_code_enum NOT NULL,
    -- last peer that got the error, used to detect when it announces successfully
    peer_id BYTEA NOT NULL,
    occurrences BIGINT NOT NULL,
    first_seen_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_seen_at TIMESTAMP WITH TIME ZONE NOT NULL,
    PRIMARY KEY (user_id, info_hash),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (torrent_id) REFERENCES torrents(id) ON DELETE CASCADE
);
CREATE INDEX announce_errors_last_seen_at_index ON announce_errors (last_seen_at);
CREATE TABLE donations  (
    id BIGSERIAL PRIMARY KEY,
    donated_by_id INT NOT NULL,
    donated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    -- in dollars
    amount FLOAT NOT NULL,
    note TEXT,
    FOREIGN KEY (donated_by_id) REFERENCES users(id),
    FOREIGN KEY (created_by_id) REFERENCES users(id)
);
CREATE TYPE bonus_points_log_action_enum AS ENUM (
    'snatch_cost_deduction',
    'snatch_cost_received_as_uploader',
    'snatch_cost_received_as_seeder',
    'torrent_upload_reward',
    'torrent_request_vote_spent',
    'torrent_request_fill_reward',
    'torrent_request_vote_refund',
    'gift_sent',
    'gift_received',
    'seedtime_reward',
    'side_effect_reward',
    'shop_purchase_upload',
    'shop_purchase_freeleech_tokens',
    'shop_purchase_promotion'
);
CREATE TABLE bonus_points_logs (
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action bonus_points_log_action_enum NOT NULL,
    amount BIGINT NOT NULL,
    details TEXT,
    item_id BIGINT
);
CREATE TABLE user_edit_change_logs (
    id BIGSERIAL PRIMARY KEY,
    item_type VARCHAR(50) NOT NULL,
    item_id BIGINT NOT NULL,
    edited_by_id INT NOT NULL REFERENCES users(id),
    edited_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    edits JSONB NOT NULL
);

CREATE TYPE user_badge_type_enum AS ENUM (
    'manual',
    'torrents_uploaded',
    'forum_posts',
    'forum_threads'
);
CREATE TABLE user_badge_categories (
    id SERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL REFERENCES users(id)
);
CREATE TABLE user_badges (
    id SERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT NOT NULL,
    image_url TEXT NOT NULL,
    category_id INT NOT NULL REFERENCES user_badge_categories(id),
    badge_type user_badge_type_enum NOT NULL,
    is_secret BOOLEAN NOT NULL DEFAULT FALSE,
    revoke_when_criteria_unmet BOOLEAN NOT NULL DEFAULT FALSE,
    criteria JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL REFERENCES users(id)
);
CREATE TABLE user_earned_badges (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    badge_id INT NOT NULL REFERENCES user_badges(id) ON DELETE CASCADE,
    awarded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    awarded_by_id INT REFERENCES users(id),
    note TEXT,
    UNIQUE(user_id, badge_id)
);

CREATE TYPE site_highlight_item_type_enum AS ENUM (
    'title_group',
    'series',
    'artist'
);
CREATE TABLE site_highlights (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT NOT NULL REFERENCES users(id),
    alias VARCHAR(60) NOT NULL,
    item_type site_highlight_item_type_enum NOT NULL,
    title_group_id INT REFERENCES title_groups(id) ON DELETE CASCADE,
    series_id BIGINT REFERENCES series(id) ON DELETE CASCADE,
    artist_id BIGINT REFERENCES artists(id) ON DELETE CASCADE,
    forum_thread_id BIGINT NOT NULL REFERENCES forum_threads(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    position INT NOT NULL UNIQUE,
    CHECK (
        (item_type = 'title_group' AND title_group_id IS NOT NULL AND series_id IS NULL AND artist_id IS NULL)
     OR (item_type = 'series'      AND series_id      IS NOT NULL AND title_group_id IS NULL AND artist_id IS NULL)
     OR (item_type = 'artist'      AND artist_id      IS NOT NULL AND title_group_id IS NULL AND series_id IS NULL)
    )
);

CREATE TABLE title_group_related_threads (
    title_group_id INT NOT NULL REFERENCES title_groups(id) ON DELETE CASCADE,
    forum_thread_id BIGINT NOT NULL REFERENCES forum_threads(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT REFERENCES users(id),
    PRIMARY KEY (title_group_id, forum_thread_id)
);
CREATE TABLE series_related_threads (
    series_id BIGINT NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    forum_thread_id BIGINT NOT NULL REFERENCES forum_threads(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT REFERENCES users(id),
    PRIMARY KEY (series_id, forum_thread_id)
);
CREATE TABLE artist_related_threads (
    artist_id BIGINT NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    forum_thread_id BIGINT NOT NULL REFERENCES forum_threads(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by_id INT REFERENCES users(id),
    PRIMARY KEY (artist_id, forum_thread_id)
);


-- Views

CREATE VIEW torrents_and_reports AS
SELECT
    t.id,
    t.upload_factor,
    t.download_factor,
    t.seeders,
    t.leechers,
    t.times_completed,
    t.grabbed,
    t.edition_group_id,
    t.created_at,
    t.updated_at,
    -- Always keep the actual created_by_id for internal queries
    t.created_by_id,
    -- Add display fields that respect anonymity
    CASE
        WHEN t.uploaded_as_anonymous THEN NULL
        ELSE t.created_by_id
    END as display_created_by_id,
    CASE
        WHEN t.uploaded_as_anonymous THEN NULL
        ELSE json_build_object('id', u.id, 'username', u.username)
    END AS display_created_by,
    t.info_hash,
    t.extras,
    t.languages,
    t.release_name,
    t.release_group,
    t.description,
    t.file_amount_per_type,
    t.uploaded_as_anonymous,
    t.file_list,
    t.mediainfo,
    t.trumpable,
    t.staff_checked,
    t.container,
    t.size,
    t.duration,
    t.audio_codec,
    t.audio_bitrate,
    t.audio_bitrate_sampling,
    t.audio_channels,
    t.video_codec,
    t.features,
    t.subtitle_languages,
    t.video_resolution,
    t.video_resolution_other_x,
    t.video_resolution_other_y,
    t.extra_text,
    t.bonus_points_snatch_cost,
    t.upload_method,
    (EXISTS (
        SELECT 1
        FROM torrent_reports tr
        WHERE tr.reported_torrent_id = t.id
    )) AS reported,
    CASE
        WHEN EXISTS (SELECT 1 FROM torrent_reports WHERE reported_torrent_id = t.id) THEN json_agg(row_to_json(tr))
        ELSE '[]'::json
    END AS reports
    FROM
        torrents t
    LEFT JOIN
        torrent_reports tr ON t.id = tr.reported_torrent_id
    LEFT JOIN
        users u ON t.created_by_id = u.id
    WHERE t.deleted_at IS NULL
    GROUP BY
        t.id, u.id
    ORDER BY
        t.id;

CREATE MATERIALIZED VIEW title_group_hierarchy_lite AS
SELECT
    title_groups.id AS title_group_id,
    title_groups.name AS title_group_name,
    title_groups.covers AS title_group_covers,
    title_groups.category AS title_group_category,
    title_groups.content_type AS title_group_content_type,
    title_groups.platform AS title_group_platform,
    title_groups.original_release_date AS title_group_original_release_date,
    title_groups.original_release_date_only_year_known AS title_group_original_release_date_only_year_known,
    title_groups.external_links AS title_group_external_links,
    title_groups.name_aliases AS title_group_name_aliases,
    tg_tags.tag_ids AS title_group_tag_ids,
    tg_tags.tag_names AS title_group_tag_names,

    series.id AS title_group_series_id,
    series.name AS title_group_series_name,

    edition_groups.id AS edition_group_id,
    edition_groups.name AS edition_group_name,
    edition_groups.release_date AS edition_group_release_date,
    edition_groups.release_date_only_year_known AS edition_group_release_date_only_year_known,
    edition_groups.distributor AS edition_group_distributor,
    edition_groups.covers AS edition_group_covers,
    edition_groups.source AS edition_group_source,
    edition_groups.additional_information AS edition_group_additional_information,

    torrents.id AS torrent_id,
    torrents.created_by_id AS torrent_created_by_id,
    torrents.uploaded_as_anonymous AS torrent_uploaded_as_anonymous,
    torrents.upload_factor AS torrent_upload_factor,
    torrents.download_factor AS torrent_download_factor,
    torrents.seeders AS torrent_seeders,
    torrents.leechers AS torrent_leechers,
    torrents.times_completed AS torrent_times_completed,
    torrents.grabbed AS torrent_grabbed,
    torrents.edition_group_id AS torrent_edition_group_id,
    torrents.created_at AS torrent_created_at,
    torrents.extras AS torrent_extras,
    torrents.release_name AS torrent_release_name,
    torrents.release_group AS torrent_release_group,
    torrents.file_amount_per_type AS torrent_file_amount_per_type,
    torrents.trumpable AS torrent_trumpable,
    torrents.staff_checked AS torrent_staff_checked,
    torrents.languages AS torrent_languages,
    torrents.container AS torrent_container,
    torrents.size AS torrent_size,
    torrents.duration AS torrent_duration,
    torrents.audio_codec AS torrent_audio_codec,
    torrents.audio_bitrate AS torrent_audio_bitrate,
    torrents.audio_bitrate_sampling AS torrent_audio_bitrate_sampling,
    torrents.audio_channels AS torrent_audio_channels,
    torrents.video_codec AS torrent_video_codec,
    torrents.features AS torrent_features,
    torrents.subtitle_languages AS torrent_subtitle_languages,
    torrents.video_resolution AS torrent_video_resolution,
    torrents.video_resolution_other_x AS torrent_video_resolution_other_x,
    torrents.video_resolution_other_y AS torrent_video_resolution_other_y,
    (EXISTS (
        SELECT 1
        FROM torrent_reports tr
        WHERE tr.reported_torrent_id = torrents.id
    )) AS torrent_reported
FROM title_groups
LEFT JOIN LATERAL (
    SELECT
        COALESCE(
            ARRAY(
                SELECT tat.tag_id
                FROM title_group_applied_tags tat
                WHERE tat.title_group_id = title_groups.id
            ),
            ARRAY[]::int[]
        ) AS tag_ids,
        COALESCE(
            ARRAY(
                SELECT t.name
                FROM title_group_applied_tags tat
                JOIN title_group_tags t ON t.id = tat.tag_id
                WHERE tat.title_group_id = title_groups.id
                  AND t.deleted_at IS NULL
            ),
            ARRAY[]::text[]
        ) AS tag_names
) tg_tags ON TRUE
LEFT JOIN edition_groups ON edition_groups.title_group_id = title_groups.id
LEFT JOIN torrents ON torrents.edition_group_id = edition_groups.id AND torrents.deleted_at IS NULL
LEFT JOIN series ON series.id = title_groups.series_id;

-- the materialized view is refreshed periodically by the periodic tasks in the backend
