

export type Winner = "home"|"away"|null;

export type GameInfo = {
    date: string,
    home: string,
    home_token: string,
    away: string,
    away_token: string,
    winner: Winner,
    sig?: string,
}

export interface JsonResponse {
    status: "success"|"error",
    data?: any,
    message?: string,
}
