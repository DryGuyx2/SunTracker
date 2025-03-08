use core::fmt;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use table_extract::Table;

use lazy_static::lazy_static;

/*
pub async fn fetch_site() -> Result<String, reqwest::Error> {
    let url: String = String::from("https://stjerneskinn.com/soloppgang-trondheim-ny.htm");

    let request: Request<String> = Request::get(url).body()?;

    eprintln!("Fetching {url:?}...");

    let res = reqwest::get(url).await?;

    eprintln!("Response: {:?} {}", res.version(), res.status());
    eprintln!("Headers: {:#?}\n", res.headers());

    let body = res.text().await?;

    Ok(body)
}
*/

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

#[derive(Debug)]
pub struct MonthParseError {
    string: String,
}

impl Display for MonthParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid string name {} for Month", self.string)
    }
}

impl std::error::Error for MonthParseError {}

impl TryInto<Month> for String {
    type Error = MonthParseError;
    fn try_into(self) -> Result<Month, Self::Error> {
        match self.as_str().trim() {
            "Jan" | "january" => Ok(Month::January),
            "Feb" | "february" => Ok(Month::February),
            "Mar" | "march" => Ok(Month::March),
            "Apr" | "april" => Ok(Month::April),
            "Mai" | "may" => Ok(Month::May),
            "Juni" | "june" => Ok(Month::June),
            "Juli" | "july" => Ok(Month::July),
            "Aug" | "august" => Ok(Month::August),
            "Sep" | "september" => Ok(Month::September),
            "Okt" | "october" => Ok(Month::October),
            "Nov" | "november" => Ok(Month::November),
            "Des" | "december" => Ok(Month::December),
            &_ => Err(Self::Error { string: self }),
        }
    }
}

#[derive(Debug)]
pub struct DayRangeError {}

impl Display for DayRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Input number not in range of days in month")
    }
}

impl std::error::Error for DayRangeError {}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct TimeStamp {
    hours: u8,
    minutes: u8,
}
impl TimeStamp {
    fn new(hours: u8, minutes: u8) -> TimeStamp {
        TimeStamp { hours, minutes }
    }
}

impl Display for TimeStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.hours, self.minutes)
    }
}

lazy_static! {
    static ref MONTH_LENGTH_MAP: HashMap<Month, u8> = {
        let mut month_length_map: HashMap<Month, u8> = HashMap::new();
        month_length_map.insert(Month::January, 31);
        month_length_map.insert(Month::February, 28);
        month_length_map.insert(Month::March, 31);
        month_length_map.insert(Month::April, 30);
        month_length_map.insert(Month::May, 31);
        month_length_map.insert(Month::June, 30);
        month_length_map.insert(Month::July, 31);
        month_length_map.insert(Month::August, 31);
        month_length_map.insert(Month::September, 30);
        month_length_map.insert(Month::October, 31);
        month_length_map.insert(Month::November, 30);
        month_length_map.insert(Month::December, 31);

        month_length_map
    };
}

#[derive(Debug)]
pub enum DateParseError {
    DayRangeError(DayRangeError),
    MonthParseError(MonthParseError),
}

impl Display for DateParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DayRangeError(e) => write!(f, "{}", e),
            Self::MonthParseError(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Date {
    month: Month,
    day: u8,
}

impl Date {
    pub fn new(month: String, day: u8) -> Result<Date, DateParseError> {
        let parsed_month: Month = match month.try_into() {
            Ok(m) => m,
            Err(e) => return Err(DateParseError::MonthParseError(e)),
        };
        if &day > MONTH_LENGTH_MAP.get(&parsed_month).unwrap() || day < 1 {
            return Err(DateParseError::DayRangeError(DayRangeError {}));
        }

        Ok(Date {
            month: parsed_month,
            day,
        })
    }
}

pub fn parse_sunsets(html: &str) -> HashMap<Date, TimeStamp> {
    const SUNSET_INDEX: usize = 4;
    const DATE_INDEX: usize = 0;
    let parsed_table = Table::find_first(html).unwrap();
    let mut parsed_sunsets: HashMap<Date, TimeStamp> = HashMap::new();
    for row in &parsed_table {
        let date = parse_date(
            row.as_slice()
                .get(DATE_INDEX)
                .unwrap_or(&String::from("None"))
                .into(),
        )
        .unwrap_or((String::from("None"), 0));
        let sunset = parse_time(
            row.as_slice()
                .get(SUNSET_INDEX)
                .unwrap_or(&String::from("None"))
                .into(),
        )
        .unwrap_or((0u8, 0u8));
        if date == (String::from("None"), 0) || sunset == (0, 0) {
            continue;
        }
        parsed_sunsets.insert(
            Date::new(date.0, date.1).expect("Failed parsing date from table"),
            TimeStamp::new(sunset.0, sunset.1),
        );
    }
    parsed_sunsets
}

fn parse_time(raw_time: String) -> Option<(u8, u8)> {
    let time_regex: Regex = Regex::new(r"(?m)(?P<hours>\d{1,2}) (?P<minutes>\d{1,2})").unwrap();
    let (_, [hours, minutes]) = time_regex
        .captures(raw_time.as_str())
        .map(|caps| caps.extract())?;
    Some((hours.parse().unwrap(), minutes.parse().unwrap()))
}

fn parse_date(raw_date: String) -> Option<(String, u8)> {
    let date_regex: Regex = Regex::new(
        r"(?m)(?<Month>Jan|Feb|Mar|Apr|Mai|Juni|Juli|Aug|Sep|Okt|Nov|Des)\. (?<Day>\d{1,2})",
    )
    .unwrap();
    let (_, [month, day]) = date_regex
        .captures(raw_date.as_str())
        .map(|caps| caps.extract())?;
    Some((String::from(month), day.parse().unwrap()))
}
