mod data;

use data::{TimeStamp, Date, Month};

use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};

use std::{collections::HashMap, io::Read};

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let mut site_contents = String::new();
    std::fs::File::open("site.html").unwrap().read_to_string(&mut site_contents).unwrap();

    //let site_contents = data::fetch_site().await?;
    //let target_date: Date = Date::new(String::from("Des"), 3u8).unwrap();

    let sunsets = data::parse_sunsets(&site_contents);

    let state: AppState = AppState::new(sunsets);

    let app = Router::new()
        .route("/{month}/{day}", get(get_timestamp))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

#[derive(Clone)]
struct AppState {
    sunset_table: HashMap<Date, TimeStamp>,
}

impl AppState {
    fn new(sunset_table: HashMap<Date, TimeStamp>) -> AppState {
        AppState {sunset_table}
    }
    fn get_sunset(&self, date: &Date) -> Option<&TimeStamp> {
        self.sunset_table.get(date)
    }
}


async fn get_timestamp(State(state): State<AppState>, Path((month, day)): Path<(String, String)>) -> String {
    let parsed_day: u8 = match day.parse() {
        Ok(num) => num,
        Err(_) => return String::from("None"),
    };

    let target_date: Date = Date::new(month, parsed_day).unwrap();
    match state.get_sunset(&target_date) {
        Some(t) => t.to_string(),
        None => String::from("None"),
    }
}
