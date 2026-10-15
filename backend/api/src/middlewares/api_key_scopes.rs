use actix_web::{dev::ResourceDef, http::Method};
use arcadia_storage::models::user::APIKeyScope;
use std::sync::LazyLock;

/// Endpoints that are reachable without any authentication, so neither a JWT token nor an
/// API key is looked at. This is the single source of truth, used both by the
/// authentication middleware and by the API documentation.
const PATHS_WITHOUT_AUTHENTICATION: &[&str] = &[
    "/api/auth/login",
    "/api/auth/register",
    "/api/auth/refresh-token",
    "/api/auth/apply",
    "/api/auth/irc",
    "/api/auth/reset-password",
    // SSE streams cannot send custom headers, so the token is passed as a query parameter
    "/api/notifications/stream",
];

/// Endpoints below these prefixes are reachable without any authentication. Like the prefixes
/// of [`SCOPE_OF_PATH_PREFIX`] below, a prefix only matches whole path segments, so `/api/css`
/// does not match `/api/css-sheets`.
const PATH_PREFIXES_WITHOUT_AUTHENTICATION: &[&str] = &["/api/css"];

/// Individual endpoints that are reachable without any authentication, matched with the same
/// dynamic segments as the routes they mirror. Unlike the prefixes above, both the method and
/// the whole path must match, so the rest of the route group they belong to keeps needing
/// authentication. A `{...}` segment matches any single path segment.
///
/// `<img src>` cannot send the `Authorization` header, so the emoji image is served without
/// authentication, like the CSS routes above. This cannot be expressed as a `/api/emojis/`
/// prefix instead, as that would also let `/api/emojis/usage` through, which must stay
/// authenticated.
const ENDPOINTS_WITHOUT_AUTHENTICATION: &[(Method, &str)] =
    &[(Method::GET, "/api/emojis/{emoji_id}/image")];

/// Every endpoint of the API belongs to exactly one API key scope. The path prefixes are
/// matched from the longest to the shortest one, so a more specific prefix always wins over
/// the scope of the route group it belongs to.
///
/// Any new endpoint must be added here, otherwise it becomes unreachable with an API key.
const SCOPE_OF_PATH_PREFIX: &[(&str, APIKeyScope)] = &[
    // user
    ("/api/arcadia-settings", APIKeyScope::User),
    ("/api/auth", APIKeyScope::User),
    ("/api/conversations", APIKeyScope::User),
    ("/api/css-sheets", APIKeyScope::User),
    ("/api/donations", APIKeyScope::User),
    ("/api/emojis", APIKeyScope::User),
    ("/api/gifts", APIKeyScope::User),
    ("/api/home", APIKeyScope::User),
    ("/api/invitations", APIKeyScope::User),
    ("/api/maintenance-tools", APIKeyScope::User),
    ("/api/notifications", APIKeyScope::User),
    ("/api/search/conversations", APIKeyScope::User),
    ("/api/search/users", APIKeyScope::User),
    ("/api/shop", APIKeyScope::User),
    ("/api/site-highlights", APIKeyScope::User),
    ("/api/staff-pms", APIKeyScope::User),
    ("/api/stats", APIKeyScope::User),
    ("/api/unauthorized-access", APIKeyScope::User),
    ("/api/user-applications", APIKeyScope::User),
    ("/api/user-badge-categories", APIKeyScope::User),
    ("/api/user-badges", APIKeyScope::User),
    ("/api/user-classes", APIKeyScope::User),
    ("/api/user-edit-change-logs", APIKeyScope::User),
    ("/api/users", APIKeyScope::User),
    // torrents
    ("/api/affiliated-artists", APIKeyScope::Torrents),
    ("/api/artists", APIKeyScope::Torrents),
    ("/api/collages", APIKeyScope::Torrents),
    ("/api/edition-groups", APIKeyScope::Torrents),
    ("/api/external-sources", APIKeyScope::Torrents),
    ("/api/image-host", APIKeyScope::Torrents),
    ("/api/master-groups", APIKeyScope::Torrents),
    ("/api/search", APIKeyScope::Torrents),
    ("/api/series", APIKeyScope::Torrents),
    (
        "/api/subscriptions/artist-title-groups",
        APIKeyScope::Torrents,
    ),
    (
        "/api/subscriptions/title-group-comments",
        APIKeyScope::Torrents,
    ),
    (
        "/api/subscriptions/title-group-torrents",
        APIKeyScope::Torrents,
    ),
    ("/api/subscriptions/collages", APIKeyScope::Torrents),
    ("/api/title-group-bookmarks", APIKeyScope::Torrents),
    ("/api/title-group-tags", APIKeyScope::Torrents),
    ("/api/title-groups", APIKeyScope::Torrents),
    ("/api/torrents", APIKeyScope::Torrents),
    // requests
    (
        "/api/search/torrent-request-comments",
        APIKeyScope::Requests,
    ),
    ("/api/search/torrent-requests", APIKeyScope::Requests),
    (
        "/api/subscriptions/torrent-request-comments",
        APIKeyScope::Requests,
    ),
    ("/api/torrent-requests", APIKeyScope::Requests),
    // forum
    ("/api/forum", APIKeyScope::Forum),
    ("/api/related-forum-threads", APIKeyScope::Forum),
    ("/api/search/forum", APIKeyScope::Forum),
    (
        "/api/subscriptions/forum-sub-category-threads",
        APIKeyScope::Forum,
    ),
    ("/api/subscriptions/forum-thread-posts", APIKeyScope::Forum),
    // wiki
    ("/api/search/wiki", APIKeyScope::Wiki),
    ("/api/wiki", APIKeyScope::Wiki),
];

/// Individual endpoints that every API key can reach, whatever its scopes are. Both the
/// method and the whole path must match, so the rest of the route group keeps needing the
/// scope it belongs to. A `{...}` segment matches any single path segment.
const ENDPOINTS_ALLOWED_FOR_EVERY_SCOPE: &[(Method, &str)] = &[
    (Method::GET, "/api/arcadia-settings/public"),
    (Method::GET, "/api/torrents/upload-info"),
];

/// Endpoints that no API key can reach, whatever its scopes are, as managing API keys would
/// let a key grant itself more scopes, and authorizing OAuth2 clients would let a stolen key
/// log into the external services (e.g. Elysia) as the owner of the key.
const PATH_PREFIXES_FORBIDDEN_FOR_EVERY_SCOPE: &[&str] =
    &["/api/users/api-keys", "/api/auth/oauth-authorize"];

/// Individual endpoints that no API key can reach, whatever its scopes are. Unlike the
/// prefixes above, both the method and the whole path must match, so the rest of the route
/// group stays reachable. A `{...}` segment matches any single path segment.
///
/// `PUT /api/users` changes the email of the account without asking for the current
/// password, so a stolen key would be enough to take the account over.
///
/// `POST /api/users/{id}/password-reset-token` returns a live password reset token, which the
/// unauthenticated `POST /api/auth/reset-password` then turns into a full account takeover.
///
/// `PUT /api/users/{id}/permissions` lets the owner of the key grant themselves any
/// permission, including the one needed by the password reset token endpoint above.
const ENDPOINTS_FORBIDDEN_FOR_EVERY_SCOPE: &[(Method, &str)] = &[
    (Method::PUT, "/api/users"),
    (Method::POST, "/api/users/{id}/password-reset-token"),
    (Method::PUT, "/api/users/{id}/permissions"),
];

/// The paths above are matched with the router of actix, so they support the same dynamic
/// segments as the routes they forbid. The authentication middleware wraps the whole `/api`
/// scope, so it runs before actix resolves the route and only has the path to match on.
static FORBIDDEN_ENDPOINT_MATCHERS: LazyLock<Vec<(Method, ResourceDef)>> =
    LazyLock::new(|| endpoint_matchers(ENDPOINTS_FORBIDDEN_FOR_EVERY_SCOPE));

/// Same as [`FORBIDDEN_ENDPOINT_MATCHERS`], for the endpoints every API key can reach.
static SCOPELESS_ENDPOINT_MATCHERS: LazyLock<Vec<(Method, ResourceDef)>> =
    LazyLock::new(|| endpoint_matchers(ENDPOINTS_ALLOWED_FOR_EVERY_SCOPE));

/// Same as [`FORBIDDEN_ENDPOINT_MATCHERS`], for the endpoints reachable without authentication.
static UNAUTHENTICATED_ENDPOINT_MATCHERS: LazyLock<Vec<(Method, ResourceDef)>> =
    LazyLock::new(|| endpoint_matchers(ENDPOINTS_WITHOUT_AUTHENTICATION));

fn endpoint_matchers(endpoints: &[(Method, &str)]) -> Vec<(Method, ResourceDef)> {
    endpoints
        .iter()
        .map(|(method, path)| (method.clone(), ResourceDef::new(*path)))
        .collect()
}

/// Checks whether the method and path of a request match one of the given endpoints.
fn matches_endpoint(matchers: &[(Method, ResourceDef)], method: &Method, path: &str) -> bool {
    matchers.iter().any(|(endpoint_method, endpoint_path)| {
        endpoint_method == method && endpoint_path.is_match(path)
    })
}

/// What an API key can do with a given endpoint.
#[derive(Debug, PartialEq, Eq)]
pub enum APIKeyAccess {
    /// Only keys granted this scope can reach the endpoint.
    Scope(APIKeyScope),
    /// Every API key can reach the endpoint, whatever its scopes are. Authentication is still
    /// required, so an anonymous request is rejected.
    EveryScope,
    /// No API key can reach the endpoint, whatever its scopes are.
    Forbidden,
    /// The endpoint is reachable without authenticating at all.
    NoAuthenticationRequired,
    /// The path does not belong to any scope, so no API key can reach it.
    Unmapped,
}

/// Checks whether the given method and path are reachable without authenticating at all.
pub fn requires_no_authentication(method: &Method, path: &str) -> bool {
    PATHS_WITHOUT_AUTHENTICATION.contains(&path)
        || PATH_PREFIXES_WITHOUT_AUTHENTICATION
            .iter()
            .any(|prefix| matches_prefix(path, prefix))
        || matches_endpoint(&UNAUTHENTICATED_ENDPOINT_MATCHERS, method, path)
}

/// Returns what an API key can do with the given endpoint. This is the single source of truth,
/// used both to authorize the requests and to document the scopes in the API documentation.
pub fn api_key_access_of_endpoint(method: &Method, path: &str) -> APIKeyAccess {
    if requires_no_authentication(method, path) {
        return APIKeyAccess::NoAuthenticationRequired;
    }

    let is_forbidden = PATH_PREFIXES_FORBIDDEN_FOR_EVERY_SCOPE
        .iter()
        .any(|prefix| matches_prefix(path, prefix))
        || matches_endpoint(&FORBIDDEN_ENDPOINT_MATCHERS, method, path);

    if is_forbidden {
        return APIKeyAccess::Forbidden;
    }

    if matches_endpoint(&SCOPELESS_ENDPOINT_MATCHERS, method, path) {
        return APIKeyAccess::EveryScope;
    }

    // the longest matching prefix wins, so a more specific prefix always overrides the scope
    // of the route group it belongs to
    SCOPE_OF_PATH_PREFIX
        .iter()
        .filter(|(prefix, _)| matches_prefix(path, prefix))
        .max_by_key(|(prefix, _)| prefix.len())
        .map_or(APIKeyAccess::Unmapped, |(_, scope)| {
            APIKeyAccess::Scope(scope.clone())
        })
}

/// Checks whether an API key granted the given scopes is allowed to reach the given endpoint.
pub fn is_endpoint_allowed_for_scopes(method: &Method, path: &str, scopes: &[APIKeyScope]) -> bool {
    match api_key_access_of_endpoint(method, path) {
        APIKeyAccess::Scope(scope) => scopes.contains(&scope),
        APIKeyAccess::EveryScope | APIKeyAccess::NoAuthenticationRequired => true,
        APIKeyAccess::Forbidden | APIKeyAccess::Unmapped => false,
    }
}

/// A prefix only matches a whole path segment, so `/api/users` does not match `/api/user-badges`.
fn matches_prefix(path: &str, prefix: &str) -> bool {
    path == prefix
        || path
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn access_of_path(path: &str) -> APIKeyAccess {
        api_key_access_of_endpoint(&Method::GET, path)
    }

    const EVERY_SCOPE: &[APIKeyScope] = &[
        APIKeyScope::User,
        APIKeyScope::Torrents,
        APIKeyScope::Requests,
        APIKeyScope::Forum,
        APIKeyScope::Wiki,
    ];

    #[test]
    fn scopes_are_resolved_with_the_longest_matching_prefix() {
        assert_eq!(
            access_of_path("/api/search"),
            APIKeyAccess::Scope(APIKeyScope::Torrents)
        );
        assert_eq!(
            access_of_path("/api/search/torrents/lite"),
            APIKeyAccess::Scope(APIKeyScope::Torrents)
        );
        assert_eq!(
            access_of_path("/api/search/wiki"),
            APIKeyAccess::Scope(APIKeyScope::Wiki)
        );
        assert_eq!(
            access_of_path("/api/search/users"),
            APIKeyAccess::Scope(APIKeyScope::User)
        );
        assert_eq!(
            access_of_path("/api/subscriptions/forum-thread-posts"),
            APIKeyAccess::Scope(APIKeyScope::Forum)
        );
        assert_eq!(access_of_path("/api/unknown"), APIKeyAccess::Unmapped);
    }

    #[test]
    fn prefixes_only_match_whole_path_segments() {
        assert_eq!(
            access_of_path("/api/users/me"),
            APIKeyAccess::Scope(APIKeyScope::User)
        );
        assert_eq!(
            access_of_path("/api/torrents-something"),
            APIKeyAccess::Unmapped,
            "a prefix must not match a longer path segment"
        );
    }

    #[test]
    fn api_key_management_is_never_reachable_with_an_api_key() {
        assert!(!is_endpoint_allowed_for_scopes(
            &Method::GET,
            "/api/users/api-keys",
            EVERY_SCOPE
        ));
        assert!(!is_endpoint_allowed_for_scopes(
            &Method::DELETE,
            "/api/users/api-keys/3",
            EVERY_SCOPE
        ));
        assert!(is_endpoint_allowed_for_scopes(
            &Method::GET,
            "/api/users/me",
            EVERY_SCOPE
        ));
    }

    #[test]
    fn editing_the_account_is_never_reachable_with_an_api_key() {
        assert!(!is_endpoint_allowed_for_scopes(
            &Method::PUT,
            "/api/users",
            EVERY_SCOPE
        ));
        assert!(
            is_endpoint_allowed_for_scopes(&Method::GET, "/api/users", EVERY_SCOPE),
            "only the method that edits the account is forbidden"
        );
        assert!(
            is_endpoint_allowed_for_scopes(&Method::PUT, "/api/users/settings", EVERY_SCOPE),
            "the rest of the route group stays reachable"
        );
    }

    #[test]
    fn taking_the_account_over_is_never_reachable_with_an_api_key() {
        assert!(!is_endpoint_allowed_for_scopes(
            &Method::POST,
            "/api/users/3/password-reset-token",
            EVERY_SCOPE
        ));
        assert!(!is_endpoint_allowed_for_scopes(
            &Method::PUT,
            "/api/users/3/permissions",
            EVERY_SCOPE
        ));
        assert!(
            is_endpoint_allowed_for_scopes(&Method::GET, "/api/users/3/permissions", EVERY_SCOPE),
            "only the method that edits the permissions is forbidden"
        );
        assert!(
            is_endpoint_allowed_for_scopes(&Method::PUT, "/api/users/3/class", EVERY_SCOPE),
            "a dynamic segment must not match another endpoint of the route group"
        );
        assert_eq!(
            api_key_access_of_endpoint(&Method::PUT, "/api/users/{id}/permissions"),
            APIKeyAccess::Forbidden,
            "the API documentation passes the templated path, which must be forbidden too"
        );
    }

    #[test]
    fn some_endpoints_are_reachable_with_any_scope() {
        assert_eq!(
            api_key_access_of_endpoint(&Method::GET, "/api/arcadia-settings/public"),
            APIKeyAccess::EveryScope
        );
        assert_eq!(
            api_key_access_of_endpoint(&Method::GET, "/api/torrents/upload-info"),
            APIKeyAccess::EveryScope
        );
        assert!(is_endpoint_allowed_for_scopes(
            &Method::GET,
            "/api/torrents/upload-info",
            &[APIKeyScope::Forum]
        ));
        assert!(
            is_endpoint_allowed_for_scopes(&Method::GET, "/api/arcadia-settings/public", &[]),
            "a key granted no scope at all still reaches those endpoints"
        );
        assert_eq!(
            api_key_access_of_endpoint(&Method::GET, "/api/arcadia-settings"),
            APIKeyAccess::Scope(APIKeyScope::User),
            "the rest of the route group still needs its scope"
        );
        assert_eq!(
            api_key_access_of_endpoint(&Method::PUT, "/api/arcadia-settings/public"),
            APIKeyAccess::Scope(APIKeyScope::User),
            "only the documented method is reachable with any scope"
        );
    }

    #[test]
    fn a_key_cannot_reach_a_scope_it_was_not_granted() {
        assert!(is_endpoint_allowed_for_scopes(
            &Method::GET,
            "/api/torrents",
            &[APIKeyScope::Torrents]
        ));
        assert!(!is_endpoint_allowed_for_scopes(
            &Method::GET,
            "/api/forum/threads",
            &[APIKeyScope::Torrents]
        ));
    }

    #[test]
    fn unauthenticated_endpoints_need_no_scope() {
        assert_eq!(
            access_of_path("/api/auth/login"),
            APIKeyAccess::NoAuthenticationRequired
        );
        assert_eq!(
            access_of_path("/api/css/dark.css"),
            APIKeyAccess::NoAuthenticationRequired
        );
        assert_eq!(
            access_of_path("/api/auth/logout"),
            APIKeyAccess::Scope(APIKeyScope::User),
            "the rest of the route group still needs authentication"
        );
    }

    #[test]
    fn only_the_emoji_image_endpoint_is_reachable_without_authentication() {
        assert_eq!(
            access_of_path("/api/emojis/102/image"),
            APIKeyAccess::NoAuthenticationRequired
        );
        assert_eq!(
            access_of_path("/api/emojis"),
            APIKeyAccess::Scope(APIKeyScope::User),
            "the templated endpoint must not let /api/emojis itself through"
        );
        assert_eq!(
            access_of_path("/api/emojis/102"),
            APIKeyAccess::Scope(APIKeyScope::User),
            "only the /image sub-path is unauthenticated"
        );
        assert_eq!(
            access_of_path("/api/emojis/usage"),
            APIKeyAccess::Scope(APIKeyScope::User),
            "a /api/emojis/ prefix would also have let this sibling route through, it must stay authenticated"
        );
        assert_eq!(
            api_key_access_of_endpoint(&Method::POST, "/api/emojis/102/image"),
            APIKeyAccess::Scope(APIKeyScope::User),
            "only GET is exempted, the rest of the route group still needs authentication"
        );
    }

    #[test]
    fn a_prefix_without_a_trailing_slash_does_not_match_a_longer_segment() {
        assert_eq!(
            access_of_path("/api/css-sheets/1"),
            APIKeyAccess::Scope(APIKeyScope::User),
            "/api/css must not match /api/css-sheets, which needs authentication"
        );
    }

    #[test]
    fn every_documented_endpoint_belongs_to_a_scope() {
        use utoipa::OpenApi;

        let openapi = crate::api_doc::ApiDoc::openapi();
        let unmapped_paths: Vec<&String> = openapi
            .paths
            .paths
            .keys()
            .filter(|path| path.starts_with("/api/"))
            .filter(|path| access_of_path(path) == APIKeyAccess::Unmapped)
            .collect();

        assert!(
            unmapped_paths.is_empty(),
            "those endpoints belong to no API key scope, add them to SCOPE_OF_PATH_PREFIX: {unmapped_paths:?}"
        );
    }
}
