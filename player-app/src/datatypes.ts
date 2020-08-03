export type Player {
  id: string;
  name: string;
  pictureUrl: string;
  balance: number;
}

export type Opponent {
  id: string;
  name: string;
  pictureUrl: string;
}

export type Challenge {
  id: string;
  opponentId: string;
  pot: number;
  status: string;
}
