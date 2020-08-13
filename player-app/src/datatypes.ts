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
// following, funding tx may or may not be mined and only players signing
  Unsigned,
  Issued,
  Received,
  Accepted,
// arbiter signed and funding tx not in chain
  Certified,
// arbiter signed and funding tx is in chain
  Live,
// local player submitted signed payout request
  PayoutRequestIssued,
// opponent submitted signed payout request
  PayoutRequestReceived,
// both players signed payout request
  PayoutRequestLive,
// payout request broadcast
  Resolved,
  Invalid,
}

export enum PayoutRequestStatus {
// the following assume the payout tx has not been mined
  Unsigned,
  LocalPlayerSigned,
  OtherPlayerSigned,
// 2/3 sigs provided
// the payout request only requires 2/3 sigs instead of 3/3 like a challenge
  Live,
// payout tx mined
  Resolved,
// if only the arbiter has signed, invalid
  Invalid,
}
