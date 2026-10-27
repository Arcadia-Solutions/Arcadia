use arcadia_common::error::Error;
use arcadia_storage::{
    models::advent_calendar::{
        AdventCalendarDoorSettings, AdventCalendarLateOpeningPolicy,
        AdventCalendarRewardDistribution, UserCreatedAdventCalendar, UserCreatedAdventCalendarDoor,
    },
    services::advent_calendar_service::{
        door_openable_until, door_windows, validate_advent_calendar,
    },
};
use chrono::{Duration, TimeZone, Utc};

#[test]
fn test_door_windows_chain_durations() {
    let start = Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).unwrap();
    let windows = door_windows(start, [1440, 60, 1440]);
    assert_eq!(
        windows,
        vec![
            (start, start + Duration::hours(24)),
            (start + Duration::hours(24), start + Duration::hours(25)),
            (start + Duration::hours(25), start + Duration::hours(49)),
        ]
    );
}

#[test]
fn test_door_openable_until_per_policy() {
    let door_end = Utc.with_ymd_and_hms(2026, 12, 2, 0, 0, 0).unwrap();
    let event_end = Utc.with_ymd_and_hms(2026, 12, 25, 0, 0, 0).unwrap();
    assert_eq!(
        door_openable_until(
            door_end,
            event_end,
            AdventCalendarLateOpeningPolicy::DuringDoorWindow,
            6
        ),
        door_end
    );
    assert_eq!(
        door_openable_until(
            door_end,
            event_end,
            AdventCalendarLateOpeningPolicy::GracePeriod,
            6
        ),
        door_end + Duration::hours(6)
    );
    assert_eq!(
        door_openable_until(
            door_end,
            event_end,
            AdventCalendarLateOpeningPolicy::UntilEventEnd,
            6
        ),
        event_end
    );
}

fn calendar_with_one_door() -> UserCreatedAdventCalendar {
    UserCreatedAdventCalendar {
        name: "Advent 2026".to_string(),
        starts_at: Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).unwrap(),
        top_message: String::new(),
        button_text: "Advent calendar".to_string(),
        staff_notes: String::new(),
        late_opening_policy: AdventCalendarLateOpeningPolicy::DuringDoorWindow,
        late_opening_grace_hours: 0,
        published: false,
        doors: vec![UserCreatedAdventCalendarDoor {
            id: None,
            settings: AdventCalendarDoorSettings {
                name: "Day 1".to_string(),
                image_url: None,
                duration_minutes: 1440,
                opening_message: String::new(),
                locked_message: None,
                passphrase: None,
                staff_notes: String::new(),
                upload_min: 0,
                upload_max: 0,
                upload_distribution: AdventCalendarRewardDistribution::Linear,
                download_removed_min: 0,
                download_removed_max: 0,
                download_removed_distribution: AdventCalendarRewardDistribution::Linear,
                freeleech_tokens_min: 0,
                freeleech_tokens_max: 0,
                freeleech_tokens_distribution: AdventCalendarRewardDistribution::Linear,
                invitations_min: 0,
                invitations_max: 0,
                invitations_distribution: AdventCalendarRewardDistribution::Linear,
                bonus_points_min: 0,
                bonus_points_max: 0,
                bonus_points_distribution: AdventCalendarRewardDistribution::Linear,
            },
        }],
    }
}

#[test]
fn test_validate_advent_calendar_rejects_bad_rewards() {
    let mut logarithmic_from_zero = calendar_with_one_door();
    logarithmic_from_zero.doors[0].settings.bonus_points_max = 10;
    logarithmic_from_zero.doors[0]
        .settings
        .bonus_points_distribution = AdventCalendarRewardDistribution::Logarithmic;
    let mut inverted_bounds = calendar_with_one_door();
    inverted_bounds.doors[0].settings.bonus_points_min = 10;
    inverted_bounds.doors[0].settings.bonus_points_max = 5;
    let mut too_many_invitations = calendar_with_one_door();
    too_many_invitations.doors[0].settings.invitations_max = 101;
    let mut no_doors = calendar_with_one_door();
    no_doors.doors.clear();
    for invalid in [
        logarithmic_from_zero,
        inverted_bounds,
        too_many_invitations,
        no_doors,
    ] {
        assert!(matches!(
            validate_advent_calendar(&invalid),
            Err(Error::InvalidAdventCalendar(_))
        ));
    }

    let mut valid = calendar_with_one_door();
    valid.doors[0].settings.invitations_min = 1;
    valid.doors[0].settings.invitations_max = 3;
    valid.doors[0].settings.invitations_distribution = AdventCalendarRewardDistribution::Gaussian;
    assert!(validate_advent_calendar(&valid).is_ok());
}

#[test]
fn test_validate_advent_calendar_caps_durations() {
    let mut too_long_door = calendar_with_one_door();
    too_long_door.doors[0].settings.duration_minutes = 525_601;
    assert!(matches!(
        validate_advent_calendar(&too_long_door),
        Err(Error::InvalidAdventCalendar(_))
    ));

    let mut too_long_grace_period = calendar_with_one_door();
    too_long_grace_period.late_opening_grace_hours = 8_761;
    assert!(matches!(
        validate_advent_calendar(&too_long_grace_period),
        Err(Error::InvalidAdventCalendar(_))
    ));
}
