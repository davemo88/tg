export interface Player {
  id:               string;
  name:             string;
  pictureUrl:       string;
}

export interface LocalPlayer {
  id:               string;
  playerId:         string;
  balance:          number;
}

export interface Challenge {
  id:               string;
  playerOneId:      string;
  PlayerTwoId:      string;
  pot:              number;
  fundingTx:        bool;
  playerOneSig:     bool;
  playerTwoSig:     bool;
  arbiterSig:       bool;
}

export interface PayoutRequest {
  id:               string;
  challengeId:      string;
  payoutTx:         bool;
  playerOneSig:     bool;
  playerTwoSig:     bool;
  arbiterSig:       bool;
  payoutToken:      bool;
}

export enum ChallengeStatus {
  Unsigned,
  Issued,
  Received,
  Accepted,
  Certified,
  Live,
  PayoutRequested,
  Resolved, 
  Invalid,
}

