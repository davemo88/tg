

export type Winner = "home"|"away"|null;

export type RelativeLoc = "home"|"away";

export type Team = {
    id: number,
    name: string,
    location: string,
}

export type GameInfo = {
    date: string,
    home: Team,
    home_token: string,
    away: Team,
    away_token: string,
    winner: Winner,
    sig: string | null,
}

export type Outcome = {
    desc: string,
    token: string,
    sig?: string,
}

export type Event = {
    desc: string,
    outcomes: Outcome[],
    pubkey: string,
}

export const toEvent = (pubkey: string, info: GameInfo) => {
    let home_wins: Outcome = {
        desc: `${info.home.name} win`,
        token: info.home_token,
    };
    let away_wins: Outcome = {
        desc: `${info.away.name} win`,
        token: info.away_token,
    };
    if ((info.winner === "home") && (info.sig !== null)) {
        home_wins.sig = info.sig;
    } else if ((info.winner === "away") && (info.sig !== null)) {
        away_wins.sig = info.sig;
    }
    let event: Event = {
        desc: `${info.away.name} at ${info.home.name} on ${info.date}`,
        outcomes: [home_wins, away_wins],
        pubkey,
    }
    return event
}

export interface JsonResponse {
    status: "success"|"error",
    data?: any,
    message?: string,
}
