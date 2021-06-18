

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

export type TokenMetadata = {
    token: string,
    desc: string,
}

export type EventMetadata = {
    desc: string,
    tokenMetadata: TokenMetadata[],
}

export interface JsonResponse {
    status: "success"|"error",
    data?: any,
    message?: string,
}
