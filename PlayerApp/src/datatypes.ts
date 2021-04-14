export interface Player {
  name:             string;
  mine:             boolean;
}

export interface Contract {
    cxid:               string;
    playerOneName:      string;
    PlayerTwoName:      string;
    amount:             number;
    desc:               string; 
    playerOneSig:       boolean;
    playerTwoSig:       boolean;
    arbiterSig:         boolean;
    fundingTx:          boolean;
    payoutTx:           boolean;
}

export interface Payout {
// TODO: https://redux-toolkit.js.org/api/createEntityAdapter#selectid
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
// selected player submitted signed payout
  PayoutIssued,
// opponent submitted signed payout
  PayoutReceived,
// both players signed payout
  PayoutLive,
// payout broadcast
  Resolved,
  Invalid,
}

export enum PayoutStatus {
// the following assume the payout tx has not been mined
  Unsigned,
  SelectedPlayerSigned,
  OtherPlayerSigned,
// 2/3 sigs provided
// the payout only requires 2/3 sigs instead of 3/3 like a contract
  Live,
// payout tx mined
  Resolved,
// if only the arbiter has signed, invalid
  Invalid,
}
