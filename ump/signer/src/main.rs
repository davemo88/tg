use reqwest;
use serde::Deserialize;
use chrono::{offset::TimeZone, Date, Local, NaiveDate};

const BASE_URL: &'static str = "https://statsapi.mlb.com/api";
const VERSION: &'static str = "1";
const ORIOLES_TEAM_ID: u64 = 110; 
//const ORIOLES_LEAGUE_ID: u64 = 103; 

const SPORT_ID: u64 = 1;
/*
    "schedule": {
        "url": BASE_URL + "{ver}/schedule",
        "path_params": {
            "ver": {
                "type": "str",
                "default": "v1",
                "leading_slash": False,
                "trailing_slash": False,
                "required": True,
            }
        },
        "query_params": [
            "scheduleType",
            "eventTypes",
            "hydrate",
            "teamId",
            "leagueId",
            "sportId",
            "gamePk",
            "gamePks",
            "venueIds",
            "gameTypes",
            "date",
            "startDate",
            "endDate",
            "opponentId",
            "fields",
        ],
        "required_params": [["sportId"], ["gamePk"], ["gamePks"]],
    }
*/

fn get_schedule (
//    team: Option<u64>,
    start_date: Date<Local>,
    end_date: Date<Local>,
//    opponent: Option<u64>,
//    game_id: Option<u64>
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let url = format!("{}/v{}/schedule/?sportId={}&teamId={}&startDate={}&endDate={}", BASE_URL, VERSION, SPORT_ID, ORIOLES_TEAM_ID, 
        format_date(&start_date), 
        format_date(&end_date));
    reqwest::blocking::get(url)
}

fn format_date(date: &Date<Local>) -> String {
    date.format("%m/%d/%Y").to_string()
}

#[derive(Debug, Deserialize, Clone)]
struct Schedule {
    dates: Vec<MlbDate>,
}

#[derive(Debug, Deserialize, Clone)]
struct MlbDate {
    date: String,
    games: Vec<MlbGame>,
}

#[derive(Debug, Deserialize, Clone)]
struct MlbGame {
    teams: MlbTeams,
}

#[derive(Debug, Deserialize, Clone)]
struct MlbTeams {
    home: MlbTeam,
    away: MlbTeam,
}

#[derive(Debug, Deserialize, Clone)]
struct MlbTeam {
    team: MlbTeamDetails,
    #[serde(rename = "isWinner")]
    is_winner: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
struct MlbTeamDetails {
    id: u64,
    name: String,
}

#[derive(Debug)]
struct GameWinner {
    home: String,
    away: String,
    date: Date<Local>,
    winner: Winner,
}

#[derive(Debug)]
enum Winner {
    Home,
    Away,
}

fn get_game_winners(schedule: Schedule) -> Vec<Option<GameWinner>> {
    schedule.dates.iter().cloned().flat_map(|date| {
        date.games.iter().map(|game| {
            let winner = if let Some(true) = game.teams.home.is_winner {
                Winner::Home
            } else if let Some(true) = game.teams.away.is_winner {
                Winner::Away
            } else {
                return None
            };

            Some(GameWinner {
                home: game.teams.home.team.name.clone(),
                away: game.teams.away.team.name.clone(),
                date: Local.from_utc_date(&NaiveDate::parse_from_str(&date.date, "%Y-%m-%d").unwrap()),
                winner,
            })

        }).collect::<Vec<Option<GameWinner>>>()
    }).filter(|gw| gw.is_some()).collect()
}

fn main() {

    let today = Local::today();
    let yesterday = today - chrono::Duration::days(1);

    let response = get_schedule(yesterday, today).unwrap().text().unwrap();

    let schedule: Schedule = serde_json::from_str(&response).unwrap();

    println!("{:?}", schedule);

    let game_winners = get_game_winners(schedule);

    println!("{:?}", game_winners);
    
//    let games = 
}
