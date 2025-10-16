mod utilities;

use utilities::Template;

/// represents times after 1970
pub struct Instant {
    secs: u64,
}

impl Instant {
    pub fn now() -> Self {
        use std::time::SystemTime;

        let time = SystemTime::now();
        let since = time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let secs = since.as_secs();
        Self { secs }
    }

    pub fn from_secs(secs: u64) -> Self {
        Self { secs }
    }

    // TODO take format string
    pub fn format(&self, template: &str) -> String {
        let template = Template::new(template, '%');
        let secs = self.secs;

        let offset_year = secs / NON_LEAP_YEAR;
        let total_days = secs / DAY;
        let mut day_of_year = total_days - offset_year * 365;

        let leap_years = (BASE_YEAR..)
            .take(offset_year as usize)
            .filter(|year| is_leap_year(*year))
            .count() as u64;
        day_of_year -= leap_years;

        let year = BASE_YEAR + offset_year;

        let date_prefixes = if is_leap_year(year) {
            LEAP_YEAR_MONTHS_PREFIX_SUM
        } else {
            NON_LEAP_YEAR_MONTHS_PREFIX_SUM
        };

        let result = date_prefixes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, (_, acc))| *acc <= day_of_year);
        let Some((month, (month_name, day_sum))) = result else {
            panic!("bad day of year {day_of_year}");
        };

        let date = day_of_year - day_sum + 1; // days are one indexed
        let month = month + 1;

        // {
        //     let first_day_of_year = (total_days - day_of_year + 3) % 7;
        //     let weeks = (day_of_year + first_day_of_year) / 7 + 1; // weeks are one indexed
        //     let first_day_of_year = DAYS[first_day_of_year as usize];
        // }

        template.interpolate(|item| {
            use std::borrow::Cow;

            match item {
                "second" => Cow::Owned(format!("{second:02}", second = secs % 60)),
                "minute" => Cow::Owned(format!("{minute:02}", minute = (secs / MINUTE) % 60)),
                // hours are one indexed
                "hour" | "hour24" => {
                    Cow::Owned(format!("{hour:02}", hour = (secs / HOUR) % 24 + 1))
                }
                // hours are one indexed
                "hour12" => Cow::Owned(format!("{hour:02}", hour = (secs / HOUR) % 12 + 1)),
                "week_day" => Cow::Borrowed(DAYS[(total_days as usize + 3) % 7]),
                "week_day_short" => Cow::Borrowed(&DAYS[(total_days as usize + 3) % 7][..3]),
                "date_suffix" => Cow::Borrowed(number_index_suffix(date as usize)),
                "date" => Cow::Owned(format!("{date:02}")),
                "month_name" => Cow::Borrowed(month_name),
                "month_name_short" => Cow::Borrowed(&month_name[..3]),
                "month" => Cow::Owned(format!("{month:02}")),
                "full_year" => Cow::Owned(format!("{year}", year = year % 100)),
                "year" => Cow::Owned(format!("{year}")),
                name => {
                    panic!("unknown interpolation {name}");
                }
            }
        })
    }

    pub fn seconds(&self) -> u64 {
        self.secs
    }

    /// `month` and `day` are one indexed
    pub fn new(year: u64, month: u64, day: u64, hour: u64, minute: u64, second: u64) -> Self {
        let date_prefixes = if is_leap_year(year) {
            LEAP_YEAR_MONTHS_PREFIX_SUM
        } else {
            NON_LEAP_YEAR_MONTHS_PREFIX_SUM
        };

        let year = year - BASE_YEAR;

        let leap_years = (BASE_YEAR..)
            .take(year as usize)
            .filter(|year| is_leap_year(*year))
            .count() as u64;
        let year = year * NON_LEAP_YEAR + leap_years * DAY;

        let days = (day - 1 + date_prefixes[month as usize - 1].1) * DAY;
        Self::from_secs(year + days + hour * HOUR + minute * MINUTE + second)
    }

    // *nth month year*
    pub fn parse_english(on: &str) -> Result<Self, &str> {
        let Some((date, rest)) = on.split_once(' ') else {
            return Err(on);
        };
        let Some((month, year)) = rest.split_once(' ') else {
            return Err(on);
        };
        let date = {
            let suffixed = date.ends_with("st")
                || date.ends_with("nd")
                || date.ends_with("rd")
                || date.ends_with("th");
            if suffixed {
                &date[..(date.len() - 2)]
            } else {
                date
            }
        };

        let Ok(year) = year.parse() else {
            return Err(on);
        };

        let months = if is_leap_year(year) {
            LEAP_YEAR_MONTHS_PREFIX_SUM
        } else {
            NON_LEAP_YEAR_MONTHS_PREFIX_SUM
        };

        let month = if month.len() == 3 {
            months
                .iter()
                .position(|(name, _)| name[..3].eq_ignore_ascii_case(month))
        } else {
            months
                .iter()
                .position(|(name, _)| name.eq_ignore_ascii_case(month))
        };

        let month = if let Some(month) = month {
            month + 1
        } else {
            return Err(on);
        };

        let Ok(day) = date.parse() else {
            return Err(on);
        };

        Ok(Self::new(year, month as u64, day, 12, 0, 0))
    }

    /// Returns the [`Duration`] between dates. If other
    pub fn difference(&self, other: Instant) -> Result<Duration, Duration> {
        let difference = self.secs - other.secs;
        Ok(Duration { secs: difference })
    }
}

pub struct Duration {
    secs: u64,
}

impl Duration {
    pub fn format(&self) -> String {
        if self.secs < MINUTE {
            format!("{secs} seconds ago", secs = self.secs)
        } else if self.secs < HOUR {
            format!("{mins} minutes ago", mins = self.secs / MINUTE)
        } else if self.secs < DAY {
            format!("{hours} hours ago", hours = self.secs / HOUR)
        } else if self.secs < WEEK {
            format!("{days} days ago", days = self.secs / DAY)
        } else if self.secs < NON_LEAP_YEAR {
            format!("{weeks} weeks ago", weeks = self.secs / WEEK)
        } else {
            format!("{years} years ago", years = self.secs / NON_LEAP_YEAR)
        }
    }
}

pub const MINUTE: u64 = 60;
pub const HOUR: u64 = 60 * MINUTE;
pub const DAY: u64 = 24 * HOUR;
pub const WEEK: u64 = 7 * DAY;

const NON_LEAP_YEAR: u64 = 365 * DAY;
// const LEAP_YEAR: u64 = 364 * DAY;

const BASE_YEAR: u64 = 1970;

/*
1	January	  31
2	February  28 (29 in leap years)
3	March	  31
4	April	  30
5	May	      31
6	June	  30
7	July      31
8	August    31
9	September 30
10	October	  31
11	November  30
12	December  31
*/
const NON_LEAP_YEAR_MONTHS_PREFIX_SUM: &[(&str, u64)] = &[
    ("January", 0),
    ("February", 31),
    ("March", 59),
    ("April", 90),
    ("May", 120),
    ("June", 151),
    ("July", 181),
    ("August", 212),
    ("September", 243),
    ("October", 273),
    ("November", 304),
    ("December", 334),
];

const LEAP_YEAR_MONTHS_PREFIX_SUM: &[(&str, u64)] = &[
    ("January", 0),
    ("February", 31),
    ("March", 60),
    ("April", 91),
    ("May", 121),
    ("June", 152),
    ("July", 182),
    ("August", 213),
    ("September", 244),
    ("October", 274),
    ("November", 305),
    ("December", 335),
];

const DAYS: &[&str] = &[
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

fn is_leap_year(year: u64) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn number_index_suffix(item: usize) -> &'static str {
    match item % 10 {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    }
}

#[allow(non_snake_case)]
pub mod FORMATS {
    /// 🇬🇧
    pub const DATE_MONTH_YEAR: &str = "%date/%month/%year";
    /// 🇺🇸
    pub const MONTH_DATE_YEAR: &str = "%month/%date/%year";

    pub const DATE_NAME_MONTH_YEAR: &str = "%week_day %date%date_suffix %month_name %year";
    pub const TIME_DATE_NAME_MONTH_YEAR: &str =
        "%hour:%minute %week_day %date%date_suffix %month_name %year";
    pub const ENGLISH: &str = "%week_day the %date%date_suffix of %month_name %year";

    pub const TIME: &str = "%hour:%minute";
    pub const TIME_WITH_SECONDS: &str = "%hour:%minute:%second";

    pub const FULL_MINIMAL: &str = "%hour:%minute:%second %date/%month/%year";
}
