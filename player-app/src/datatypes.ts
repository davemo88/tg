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

export type Challenge {
  id: string;
  playerOneId: string;
  PlayerTwoId: string;
  pot: number;
  status: string;
}
