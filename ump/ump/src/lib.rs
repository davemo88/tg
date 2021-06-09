pub use bitcoin;
pub use chrono;
pub use hex;
pub use reqwest;
use std::collections::HashMap;
use bitcoin::PublicKey;
use serde::{Serialize, Deserialize};

pub const UMP_PUBKEY: &'static str = "025c571f77d693246e64f01ef740064a0b024a228813c94ae7e1e4ee73e991e0ba";

#[derive(Debug)]
pub enum BaseballGameOutcome {
    HomeWins,
    AwayWins,
    Tie,
    Cancelled,
}

impl std::fmt::Display for BaseballGameOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub home: String,
    pub away: String,
    pub date: String,
    pub outcome_tokens: HashMap<String, (i64, String, Option<String>)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonResponse<T: Serialize> {
    pub status: Status,
    pub data: Option<T>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSignatureBody {
    pub outcome_id: i64,
    pub sig_hex: String,
}

impl<T: Serialize> JsonResponse<T> {
    pub fn success(data: Option<T>) -> Self {
        JsonResponse {
            status: Status::Success,
            data,
            message: None,
        }
    }
    
    pub fn error(message: String, data: Option<T>) -> Self {
        JsonResponse {
            status: Status::Error,
            data,
            message: Some(message),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Success,
    Error,
}

pub fn ump_pubkey() -> PublicKey {
    PublicKey::from_slice(&hex::decode(UMP_PUBKEY).unwrap()).unwrap()
}

pub mod mlb_api {
    use reqwest;
    use serde::Deserialize;
    use chrono::{Date, Local};

    const BASE_URL: &'static str = "https://statsapi.mlb.com/api";
    const VERSION: &'static str = "v1";
    const SPORT_ID: i64 = 1;

    pub fn request_url(resource: &str, params: Option<&str>) -> String {
        format!("{}/{}/{}/?sportId={}{}",
            BASE_URL,
            VERSION,
            resource,
            SPORT_ID,
            params.unwrap_or_default(),
        )
    }

    /*
        "schedule": {
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

    pub fn get_schedule (
        start_date: Date<Local>,
        end_date: Date<Local>,
        team: Option<i64>,
    //    opponent: Option<i64>,
    //    game_id: Option<i64>
    ) -> Result<reqwest::blocking::Response, reqwest::Error> {
//        let url = format!("{}/v{}/schedule/?sportId={}&startDate={}&endDate={}{}", BASE_URL, VERSION, SPORT_ID, 
//            format_date(&start_date), 
//            format_date(&end_date),
//            if let Some(team) = team { format!("&teamId={}", team) } else { String::default() });
        let url = request_url(
            "schedule", 
            Some(&format!("&startDate={}&endDate={}{}",
                format_date(&start_date), 
                format_date(&end_date),
                if let Some(team) = team { format!("&teamId={}", team) } else { String::default() }))
            );

        reqwest::blocking::get(url)
    }

    pub fn get_teams() -> Result<reqwest::blocking::Response, reqwest::Error> {
        reqwest::blocking::get(request_url("teams", None))
    }
    
    fn format_date(date: &Date<Local>) -> String {
        date.format("%m/%d/%Y").to_string()
    }
    
    #[derive(Debug, Deserialize, Clone)]
    pub struct MlbSchedule {
        pub dates: Vec<MlbDate>,
    }
    
    #[derive(Debug, Deserialize, Clone)]
    pub struct MlbDate {
        pub date: String,
        pub games: Vec<MlbGame>,
    }
    
    #[derive(Debug, Deserialize, Clone)]
    pub struct MlbGame {
        #[serde(rename = "gamePk")]
        pub id: i64, 
        pub teams: MlbGameTeams,
    }
    
    #[derive(Debug, Deserialize, Clone)]
    pub struct MlbGameTeams {
        pub home: MlbGameTeam,
        pub away: MlbGameTeam,
    }
    
    #[derive(Debug, Deserialize, Clone)]
    pub struct MlbGameTeam {
        pub team: MlbTeam,
        #[serde(rename = "isWinner")]
        pub is_winner: Option<bool>,
    }
    
    #[derive(Debug, Deserialize, Clone)]
    pub struct MlbTeam {
        pub id: i64,
        pub name: String,
    }
    
    #[derive(Debug, Deserialize, Clone)]
    pub struct MlbTeams {
        pub teams: Vec<MlbTeam>,
    }

}
