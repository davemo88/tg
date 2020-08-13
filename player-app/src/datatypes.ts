export type PlayerId = string;
export type LocalPlayerId = string;
export type ChallengeId = string;
export type PayoutRequestId = string;
export type Url = string;

export interface Player {
  id:               PlayerId;
  name:             string;
  pictureUrl:       Url;
}

export interface LocalPlayer {
  id:               LocalPlayerId;
  playerId:         PlayerId;
  balance:          number;
}

export interface Challenge {
  id:               ChallengeId;
  playerOneId:      PlayerId;
  PlayerTwoId:      PlayerId;
  pot:              number;
  fundingTx:        bool;
  playerOneSig:     bool;
  playerTwoSig:     bool;
  arbiterSig:       bool;
}

export interface PayoutRequest {
  id:               PayoutRequestId;
  challengeId:      ChallengeId;
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
  PayoutRequestIssued,
  PayoutRequestReceived,
  PayoutRequestLive,
  Resolved,
  Invalid,
}

export enum PayoutRequestStatus {
  Unsigned,
  Issued,
  Received,
  Live,
  Resolved,
  Invalid,
}

