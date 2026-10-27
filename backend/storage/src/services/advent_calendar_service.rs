use crate::models::advent_calendar::{
    AdventCalendar, AdventCalendarDoorForUser, AdventCalendarDoorOpening,
    AdventCalendarDoorOpeningResult, AdventCalendarDoorState, AdventCalendarLateOpeningPolicy,
    AdventCalendarRewardDistribution, PublishedAdventCalendar, UserCreatedAdventCalendar,
};
use arcadia_common::error::{Error, Result};
use chrono::{DateTime, Duration, Utc};

// keeps the event end far from the date limits, which would break the calendar for every user
const MAXIMUM_DOOR_DURATION_MINUTES: i32 = 365 * 24 * 60;
const MAXIMUM_GRACE_HOURS: i32 = 365 * 24;
// users.invitations is a SMALLINT, a large reward would overflow it and make the door fail to open
const MAXIMUM_INVITATIONS_REWARD: i64 = 100;
// same for the other rewards, keeps the user balances far from their column limits
const MAXIMUM_FREELEECH_TOKENS_REWARD: i64 = 1_000_000;
const MAXIMUM_BYTES_REWARD: i64 = 1 << 60;
const MAXIMUM_BONUS_POINTS_REWARD: i64 = 1 << 60;

/// Chains the door durations from the calendar start, returning the (start, end) window of each door
pub fn door_windows(
    starts_at: DateTime<Utc>,
    durations_minutes: impl IntoIterator<Item = i32>,
) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    let mut door_start = starts_at;
    durations_minutes
        .into_iter()
        .map(|duration| {
            let door_end = door_start + Duration::minutes(duration as i64);
            let window = (door_start, door_end);
            door_start = door_end;
            window
        })
        .collect()
}

/// The last moment a door can be opened, depending on the calendar's late opening policy
pub fn door_openable_until(
    door_end: DateTime<Utc>,
    event_end: DateTime<Utc>,
    policy: AdventCalendarLateOpeningPolicy,
    grace_hours: i32,
) -> DateTime<Utc> {
    match policy {
        AdventCalendarLateOpeningPolicy::DuringDoorWindow => door_end,
        AdventCalendarLateOpeningPolicy::GracePeriod => {
            door_end + Duration::hours(grace_hours as i64)
        }
        AdventCalendarLateOpeningPolicy::UntilEventEnd => event_end,
    }
}

/// Checks a calendar sent by staff before it is saved
pub fn validate_advent_calendar(calendar: &UserCreatedAdventCalendar) -> Result<()> {
    let invalid = |message: String| Err(Error::InvalidAdventCalendar(message));

    if calendar.name.trim().is_empty() {
        return invalid("the name must not be empty".to_string());
    }
    if calendar.button_text.trim().is_empty() || calendar.button_text.chars().count() > 50 {
        return invalid("the button text must be between 1 and 50 characters".to_string());
    }
    if !(0..=MAXIMUM_GRACE_HOURS).contains(&calendar.late_opening_grace_hours) {
        return invalid(format!(
            "the grace period must be between 0 and {MAXIMUM_GRACE_HOURS} hours"
        ));
    }
    if calendar.doors.is_empty() {
        return invalid("the calendar must have at least one door".to_string());
    }

    for door in calendar.doors.iter().map(|door| &door.settings) {
        if door.name.trim().is_empty() {
            return invalid("every door must have a name".to_string());
        }
        if !(1..=MAXIMUM_DOOR_DURATION_MINUTES).contains(&door.duration_minutes) {
            return invalid(format!(
                "the duration of {} must be between 1 minute and 1 year",
                door.name
            ));
        }
        let rewards = [
            (
                "upload",
                door.upload_min,
                door.upload_max,
                door.upload_distribution,
                MAXIMUM_BYTES_REWARD,
            ),
            (
                "download removed",
                door.download_removed_min,
                door.download_removed_max,
                door.download_removed_distribution,
                MAXIMUM_BYTES_REWARD,
            ),
            (
                "freeleech tokens",
                door.freeleech_tokens_min,
                door.freeleech_tokens_max,
                door.freeleech_tokens_distribution,
                MAXIMUM_FREELEECH_TOKENS_REWARD,
            ),
            (
                "invitations",
                door.invitations_min,
                door.invitations_max,
                door.invitations_distribution,
                MAXIMUM_INVITATIONS_REWARD,
            ),
            (
                "bonus points",
                door.bonus_points_min,
                door.bonus_points_max,
                door.bonus_points_distribution,
                MAXIMUM_BONUS_POINTS_REWARD,
            ),
        ];
        for (reward_name, min, max, distribution, maximum_allowed) in rewards {
            if min < 0 || max < min {
                return invalid(format!(
                    "the {reward_name} bounds of {} must satisfy 0 <= min <= max",
                    door.name
                ));
            }
            if distribution == AdventCalendarRewardDistribution::Logarithmic && min < 1 {
                return invalid(format!(
                    "the logarithmic {reward_name} reward of {} must have a minimum of at least 1",
                    door.name
                ));
            }
            if max > maximum_allowed {
                return invalid(format!(
                    "the {reward_name} reward of {} is too large",
                    door.name
                ));
            }
        }
    }
    Ok(())
}

/// Builds the calendar as seen by a user: door windows, states, and the content of the doors they opened
pub fn build_published_advent_calendar(
    calendar: AdventCalendar,
    openings: &[AdventCalendarDoorOpening],
    now: DateTime<Utc>,
) -> PublishedAdventCalendar {
    let windows = door_windows(
        calendar.starts_at,
        calendar
            .doors
            .iter()
            .map(|door| door.settings.duration_minutes),
    );
    let ends_at = windows
        .last()
        .map_or(calendar.starts_at, |(_, door_end)| *door_end);

    let doors = calendar
        .doors
        .0
        .into_iter()
        .zip(windows)
        .map(|(door, (starts_at, door_end))| {
            let openable_until = door_openable_until(
                door_end,
                ends_at,
                calendar.late_opening_policy,
                calendar.late_opening_grace_hours,
            );
            let opening = openings
                .iter()
                .find(|opening| opening.door_id == door.id)
                .map(|opening| AdventCalendarDoorOpeningResult {
                    opening_message: door.settings.opening_message.clone(),
                    upload: opening.upload,
                    download_removed: opening.download_removed,
                    freeleech_tokens: opening.freeleech_tokens,
                    invitations: opening.invitations,
                    bonus_points: opening.bonus_points,
                });
            let state = if opening.is_some() {
                AdventCalendarDoorState::Opened
            } else if now < starts_at {
                AdventCalendarDoorState::Locked
            } else if now < openable_until {
                AdventCalendarDoorState::Openable
            } else {
                AdventCalendarDoorState::Missed
            };
            AdventCalendarDoorForUser {
                id: door.id,
                position: door.position,
                name: door.settings.name,
                image_url: door.settings.image_url,
                starts_at,
                ends_at: door_end,
                openable_until,
                requires_passphrase: door.settings.passphrase.is_some(),
                locked_message: door.settings.locked_message.filter(|_| {
                    door.settings.passphrase.is_some() && state == AdventCalendarDoorState::Openable
                }),
                state,
                opening,
            }
        })
        .collect();

    PublishedAdventCalendar {
        id: calendar.id,
        name: calendar.name,
        top_message: calendar.top_message,
        starts_at: calendar.starts_at,
        ends_at,
        doors,
    }
}

/// Checks that the door belongs to the calendar, can be opened right now, and that the passphrase is right
pub fn check_advent_calendar_door_openable(
    calendar: AdventCalendar,
    door_id: i32,
    given_passphrase: Option<&str>,
    now: DateTime<Utc>,
) -> Result<()> {
    let expected_passphrase = calendar
        .doors
        .iter()
        .find(|door| door.id == door_id)
        .ok_or(Error::AdventCalendarDoorNotFound)?
        .settings
        .passphrase
        .clone();
    // the openings are left out, opening the same door twice is prevented when inserting the opening
    let published_calendar = build_published_advent_calendar(calendar, &[], now);
    if published_calendar
        .doors
        .iter()
        .find(|door| door.id == door_id)
        .is_none_or(|door| door.state != AdventCalendarDoorState::Openable)
    {
        return Err(Error::AdventCalendarDoorNotOpenable);
    }
    // ignores case and surrounding whitespace
    if let Some(expected_passphrase) = expected_passphrase
        && expected_passphrase.trim().to_lowercase()
            != given_passphrase.unwrap_or_default().trim().to_lowercase()
    {
        return Err(Error::WrongAdventCalendarDoorPassphrase);
    }
    Ok(())
}
