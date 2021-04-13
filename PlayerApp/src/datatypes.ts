// for redux toolkit entity adapter
export type EntityId = string;

export interface Player {
  id:               EntityId;
  name:             string;
  mine:             boolean;
}

export interface Contract {
    id:                 EntityId;
    playerOneId:        EntityId;
    PlayerTwoId:        EntityId;
    cxid:               string;
    amount:             number;
    desc:               string; 
    playerOneSig:       boolean;
    playerTwoSig:       boolean;
    arbiterSig:         boolean;
    fundingTx:          boolean;
    payoutTx:           boolean;
}

export interface PayoutRequest {
  id:               EntityId;
  cxid:             string;
  payoutTx:         boolean;
  playerOneSig:     boolean;
  playerTwoSig:     boolean;
  arbiterSig:       boolean;
  payoutToken:      string;
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
