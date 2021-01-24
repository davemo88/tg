// for redux toolkit entity adapter
export type EntityId = string;

export interface Player {
  id:               EntityId;
  name:             string;
  pictureUrl:       string;
  mine:             bool;
}

export interface Contract {
  id:               EntityId;
  playerOneId:      EntityId;
  PlayerTwoId:      EntityId;
  cxid:             string,
  pot:              number;
  fundingTx:        bool;
  playerOneSig:     bool;
  playerTwoSig:     bool;
  arbiterSig:       bool;
}

export interface PayoutRequest {
  id:               EntityId;
  cxid:             string;
  payoutTx:         bool;
  playerOneSig:     bool;
  playerTwoSig:     bool;
  arbiterSig:       bool;
  payoutToken:      bool;
  playerOneAmount:  number;
  playerTwoAmount:  number;
}

export enum ContractStatus {
// following, funding tx may or may not be mined and only players signing
  Unsigned,
  Issued,
  Received,
  Accepted,
// arbiter signed and funding tx not in chain
  Certified,
// arbiter signed and funding tx is in chain
  Live,
// selected player submitted signed payout request
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
  SelectedPlayerSigned,
  OtherPlayerSigned,
// 2/3 sigs provided
// the payout request only requires 2/3 sigs instead of 3/3 like a contract
  Live,
// payout tx mined
  Resolved,
// if only the arbiter has signed, invalid
  Invalid,
}
