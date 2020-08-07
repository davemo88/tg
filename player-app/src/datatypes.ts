export type Player {
  id: string;
  name: string;
  pictureUrl: string;
}

export type LocalPlayer {
  id: string,
  playerId: string,
  balance: number,
}

export type Opponent {
  id: string,
  playerId: string,
}

export type Challenge {
  id: string;
  opponentId: string;
  pot: number;
  status: string;
}
