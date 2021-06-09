use ump::{
    bitcoin::{PrivateKey, secp256k1},
    chrono::{offset::TimeZone, Date, Local, NaiveDate, Duration},
    hex,
    reqwest,
    AddSignatureBody,
    BaseballGameOutcome,
    GameInfo,
    JsonResponse,
    mlb_api::{
        get_schedule,
        MlbSchedule,
    }
};

pub const UMP_PRIVKEY: &'static str = "L52hw8to1fdBj9eP8HESBNrfcbehxvKU1vsqWjmHJavxNEi9q91i";
const PUBLISHER_URL: &'static str = "http://0.0.0.0:6000";
const ORIOLES_TEAM_ID: u64 = 110; 

#[derive(Debug)]
struct GameOutcome {
    home: String,
    away: String,
    date: Date<Local>,
    outcome: BaseballGameOutcome,
}

fn get_game_outcomes(schedule: MlbSchedule) -> Vec<GameOutcome> {
    schedule.dates.iter().cloned().flat_map(|date| {
        date.games.iter().map(|game| {
            let outcome = if let Some(true) = game.teams.home.is_winner {
                BaseballGameOutcome::HomeWins
            } else if let Some(true) = game.teams.away.is_winner {
                BaseballGameOutcome::AwayWins
            } else {
                return None
            };

            Some(GameOutcome {
                home: game.teams.home.team.name.clone(),
                away: game.teams.away.team.name.clone(),
                date: Local.from_utc_date(&NaiveDate::parse_from_str(&date.date, "%Y-%m-%d").unwrap()),
                outcome,
            })

        }).collect::<Vec<Option<GameOutcome>>>()
    }).filter_map(|gw| gw).collect()
}

fn main() {
    let today = Local::today();
    let yesterday = today - Duration::days(1);

    let response = get_schedule(yesterday, today, None).unwrap().text().unwrap();

    let schedule: MlbSchedule = serde_json::from_str(&response).unwrap();

    println!("Orioles Schedule for {} - {}", today.format("%Y-%m-%d"), yesterday.format("%Y-%d-%m"));
    for date in schedule.dates.iter() {
        println!("{:?}", date);
    }

    let game_outcomes = get_game_outcomes(schedule);

    println!("Winners");
    println!("{:?}", game_outcomes);

    let response = reqwest::blocking::get(format!("{}/game-info",PUBLISHER_URL)).unwrap().text().unwrap();
    let response: JsonResponse<Vec<GameInfo>> = serde_json::from_str(&response).unwrap();

    let game_info: Vec<GameInfo> = response.data.unwrap();

    println!("Game Info");
    println!("{:?}", game_info.len());

    let new_outcomes = game_info.iter().filter_map(|info| {
        if let Some(outcome) = game_outcomes.iter().find(|outcome| {
// BUG: team names from csv don't match those from mlb api
// TODO: get all data from mlb api so this doesn't happen
            info.home == outcome.home && //.split_whitespace().last().unwrap() &&
            info.away == outcome.away && //.split_whitespace().last().unwrap() &&
            Local.from_local_date(&NaiveDate::parse_from_str(&info.date,"%Y-%m-%d").unwrap()).unwrap() == outcome.date &&
            info.outcome_tokens.values().all(|(_outcome_id, _token, sig)| sig.is_none())
        }) {
            Some((outcome, info))
        } else {
            None
        }
    }).collect::<Vec<(&GameOutcome, &GameInfo)>>();

    println!("New Winners");
    println!("{:?}", new_outcomes);

    let key = PrivateKey::from_wif(UMP_PRIVKEY).unwrap();
    let secp = secp256k1::Secp256k1::new();

    let token_sigs = new_outcomes.iter().map(|(outcome, info)| {
        let (outcome_id, token) = match info.outcome_tokens.get(&outcome.outcome.to_string()) {
            Some((outcome_id, token, _sig)) => (outcome_id,token),
            _ => panic!("outcome was {}", outcome.outcome.to_string())

        };
        println!("token to sign: {:?}", token);
        (*outcome_id, secp.sign(&secp256k1::Message::from_slice(&hex::decode(token).unwrap()).unwrap(), &key.key))
    }).collect::<Vec<(i64, secp256k1::Signature)>>();

    let client = reqwest::blocking::Client::new();
    for (outcome_id, sig) in token_sigs {
        client.post(format!("{}/signature",PUBLISHER_URL))
            .body(serde_json::to_string(&AddSignatureBody {
                outcome_id,
                sig_hex: hex::encode(sig.serialize_der().to_vec()),
            }).unwrap())
            .send()
            .unwrap();
    }
}
