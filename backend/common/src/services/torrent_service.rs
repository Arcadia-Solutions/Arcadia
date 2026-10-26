pub fn get_announce_url(passkey: &str, tracker_url: &str) -> String {
    let tracker_url = tracker_url.trim_end_matches('/');
    format!("{tracker_url}/announce/{passkey}")
}

pub fn looks_like_url(s: &str) -> bool {
    let s = s.trim();
    let b = s.as_bytes();
    (b.len() >= 7 && b[..7].eq_ignore_ascii_case(b"http://"))
        || (b.len() >= 8 && b[..8].eq_ignore_ascii_case(b"https://"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_announce_url_without_trailing_slash() {
        let url = get_announce_url("my_passkey_123", "http://localhost:8081");
        assert_eq!(url, "http://localhost:8081/announce/my_passkey_123");
    }

    #[test]
    fn test_get_announce_url_with_trailing_slash() {
        let url = get_announce_url("my_passkey_123", "http://localhost:8081/");
        assert_eq!(url, "http://localhost:8081/announce/my_passkey_123");
    }
}
