use chrono::Datelike;

use crate::config::schema::{DndSchedule, WeekDay};

/// Check if DND should be active based on current time and schedules
pub fn is_schedule_active(schedules: &[DndSchedule]) -> bool {
    let now = chrono::Local::now();
    let current_time = now.format("%H:%M").to_string();
    let current_weekday = to_chrono_weekday(&now.weekday());

    for schedule in schedules {
        if !schedule.enabled {
            continue;
        }

        // Check if today is in the schedule's days
        if !schedule.days.iter().any(|d| same_weekday(d, &current_weekday)) {
            continue;
        }

        // Check if current time is within the schedule
        if current_time >= schedule.start_time && current_time < schedule.end_time {
            return true;
        }
    }
    false
}

fn to_chrono_weekday(wd: &chrono::Weekday) -> String {
    match wd {
        chrono::Weekday::Mon => "Mon".into(),
        chrono::Weekday::Tue => "Tue".into(),
        chrono::Weekday::Wed => "Wed".into(),
        chrono::Weekday::Thu => "Thu".into(),
        chrono::Weekday::Fri => "Fri".into(),
        chrono::Weekday::Sat => "Sat".into(),
        chrono::Weekday::Sun => "Sun".into(),
    }
}

fn same_weekday(config_day: &WeekDay, chrono_day: &str) -> bool {
    let day_str = match config_day {
        WeekDay::Mon => "Mon",
        WeekDay::Tue => "Tue",
        WeekDay::Wed => "Wed",
        WeekDay::Thu => "Thu",
        WeekDay::Fri => "Fri",
        WeekDay::Sat => "Sat",
        WeekDay::Sun => "Sun",
    };
    day_str == chrono_day
}
