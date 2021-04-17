export interface Player {
  name:             string;
  mine:             boolean;
}

export interface Contract {
    cxid:               string;
    p1Name:             string;
    p2Name:             string;
    amount:             number;
    desc:               string; 
    p1Sig:              boolean;
    p2Sig:              boolean;
    arbiterSig:         boolean;
    fundingTx:          boolean;
    payoutTx:           boolean;
}

export interface Payout {
  cxid:             string;
  payoutTx:         boolean;
  p1Sig:            boolean;
  p2Sig:            boolean;
  arbiterSig:       boolean;
  payoutToken:      string;
  playerOneAmount:  number;
  playerTwoAmount:  number;
}

export enum ContractStatus {
// following, funding tx may or may not be mined and only players signing
  Unsigned,
// local player one signed
  Signed,
// non-local player one signed
  Received,
// signed by both players
  PlayersSigned,
// all signed and funding tx not in chain
  Certified,
// arbiter signed and funding tx is in chain
  Live,
// payout unsigned
  PayoutUnsigned,
// payout signed by one player
  PayoutSigned,
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
  WeSigned,
  TheySigned,
// 2/3 sigs provided
// the payout only requires 2/3 sigs instead of 3/3 like the contract
  Live,
// payout tx mined
  Resolved,
// if only the arbiter has signed, invalid
  Invalid,
}

export interface JsonResponse {
    status: "success"|"error";
    data?: any;
    message?: string;
}
