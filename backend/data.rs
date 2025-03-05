use core::fmt;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::{write, Display};
use std::hash::Hash;
use table_extract::Table;

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
pub struct MonthParseError {}

impl Display for MonthParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid string name {} for Month", self)
    }
}

impl std::error::Error for MonthParseError {}

impl TryInto<Month> for String {
    type Error = MonthParseError;
    fn try_into(self) -> Result<Month, Self::Error> {
        match self.as_str() {
            "Jan" => Ok(Month::January),
            "Feb" => Ok(Month::February),
            "Mar" => Ok(Month::March),
            "Apr" => Ok(Month::April),
            "Mai" => Ok(Month::May),
            "Juni" => Ok(Month::June),
            "Juli" => Ok(Month::July),
            "Aug" => Ok(Month::August),
            "Sep" => Ok(Month::September),
            "Okt" => Ok(Month::October),
            "Nov" => Ok(Month::November),
            "Des" => Ok(Month::December),
            &_ => Err(Self::Error {}),
        }
    }
}

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


#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Date {
    month: Month,
    day: u8,
}

impl Date {
    pub fn new(month: String, day: u8) -> Result<Date, MonthParseError> {
        Ok(Date {
            month: month.try_into()?,
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
