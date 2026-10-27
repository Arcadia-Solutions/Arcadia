use crate::models::user_badge::{UserBadgeCriteria, UserBadgeType};
use arcadia_common::error::{Error, Result};

/// Returns the parsed criteria, None for manual badges
pub fn validate_badge_criteria_shape(
    badge_type: &UserBadgeType,
    criteria: Option<&serde_json::Value>,
) -> Result<Option<UserBadgeCriteria>> {
    match (badge_type, criteria) {
        (UserBadgeType::Manual, None) => Ok(None),
        (UserBadgeType::Manual, Some(_)) => Err(Error::UserBadgeCriteriaMismatch),
        (_, None) => Err(Error::UserBadgeCriteriaMismatch),
        (badge_type, Some(value)) => {
            let parsed: UserBadgeCriteria = serde_json::from_value(value.clone())
                .map_err(|_| Error::UserBadgeCriteriaMismatch)?;
            let shapes_match = matches!(
                (badge_type, &parsed),
                (
                    UserBadgeType::TorrentsUploaded,
                    UserBadgeCriteria::TorrentsUploaded { .. }
                ) | (
                    UserBadgeType::ForumPosts,
                    UserBadgeCriteria::ForumPosts { .. }
                ) | (
                    UserBadgeType::ForumThreads,
                    UserBadgeCriteria::ForumThreads { .. }
                ) | (
                    UserBadgeType::AdventCalendarDoors,
                    UserBadgeCriteria::AdventCalendarDoors { .. }
                )
            );
            if shapes_match {
                Ok(Some(parsed))
            } else {
                Err(Error::UserBadgeCriteriaMismatch)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn torrents_uploaded_criteria() -> serde_json::Value {
        json!({
            "type": "torrents_uploaded",
            "search": {
                "title_group_include_empty_groups": false,
                "page": 1,
                "page_size": 1,
                "order_by_column": "torrent_created_at",
                "order_by_direction": "desc"
            },
            "minimum_title_group_amount": 2
        })
    }

    fn forum_posts_criteria() -> serde_json::Value {
        json!({
            "type": "forum_posts",
            "minimum_post_character_count": 30,
            "required_substring": null,
            "minimum_post_amount": 2
        })
    }

    fn forum_threads_criteria() -> serde_json::Value {
        json!({
            "type": "forum_threads",
            "minimum_thread_name_character_count": 10,
            "required_substring": null,
            "minimum_thread_amount": 1
        })
    }

    fn advent_calendar_doors_criteria() -> serde_json::Value {
        json!({
            "type": "advent_calendar_doors",
            "calendar_id": 1,
            "minimum_opened_door_amount": 24,
            "door_names": ["Day 24"]
        })
    }

    #[test]
    fn manual_without_criteria_is_ok() {
        assert!(validate_badge_criteria_shape(&UserBadgeType::Manual, None).is_ok());
    }

    #[test]
    fn manual_with_criteria_is_rejected() {
        let criteria = forum_posts_criteria();
        assert!(matches!(
            validate_badge_criteria_shape(&UserBadgeType::Manual, Some(&criteria)),
            Err(Error::UserBadgeCriteriaMismatch)
        ));
    }

    #[test]
    fn auto_badge_without_criteria_is_rejected() {
        assert!(matches!(
            validate_badge_criteria_shape(&UserBadgeType::ForumPosts, None),
            Err(Error::UserBadgeCriteriaMismatch)
        ));
    }

    #[test]
    fn matching_criteria_shape_is_ok() {
        let torrents = torrents_uploaded_criteria();
        let posts = forum_posts_criteria();
        let threads = forum_threads_criteria();
        let advent_calendar_doors = advent_calendar_doors_criteria();
        assert!(
            validate_badge_criteria_shape(&UserBadgeType::TorrentsUploaded, Some(&torrents))
                .is_ok()
        );
        assert!(validate_badge_criteria_shape(&UserBadgeType::ForumPosts, Some(&posts)).is_ok());
        assert!(
            validate_badge_criteria_shape(&UserBadgeType::ForumThreads, Some(&threads)).is_ok()
        );
        assert!(validate_badge_criteria_shape(
            &UserBadgeType::AdventCalendarDoors,
            Some(&advent_calendar_doors)
        )
        .is_ok());
    }

    #[test]
    fn mismatched_tag_is_rejected() {
        let posts = forum_posts_criteria();
        assert!(matches!(
            validate_badge_criteria_shape(&UserBadgeType::TorrentsUploaded, Some(&posts)),
            Err(Error::UserBadgeCriteriaMismatch)
        ));

        let threads = forum_threads_criteria();
        assert!(matches!(
            validate_badge_criteria_shape(&UserBadgeType::ForumPosts, Some(&threads)),
            Err(Error::UserBadgeCriteriaMismatch)
        ));
    }

    #[test]
    fn malformed_criteria_is_rejected() {
        let garbage = json!({ "type": "unknown", "foo": 1 });
        assert!(matches!(
            validate_badge_criteria_shape(&UserBadgeType::ForumPosts, Some(&garbage)),
            Err(Error::UserBadgeCriteriaMismatch)
        ));
    }
}
