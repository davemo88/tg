export interface Player {
  name:             string,
  mine:             boolean,
}

export interface Contract {
    cxid:               string,
    p1Name:             string,
    p2Name:             string,
    amount:             number,
    desc:               string, 
    p1Sig:              boolean,
    p2Sig:              boolean,
    arbiterSig:         boolean,
    txid:               string,
    txStatus:           TxStatus,
}

export interface Payout {
  cxid:             string,
  p1Amount:         number,
  p2Amount:         number,
  p1Sig:            boolean,
  p2Sig:            boolean,
  arbiterSig:       boolean,
  scriptSig:        string,
  txid:             string,
  txStatus:         TxStatus,
}

export enum TxStatus {
    Unbroadcast,
    Broadcast,
    Confirmed,
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
// all signed and funding tx not broadcast yet
  Certified,
// all signed and funding tx broadcast
  FundingTxBroadcast,
// arbiter signed and funding tx is in chain
  Live,
// payout unsigned
  PayoutUnsigned,
// payout signed by one player
  PayoutSigned,
// opponent submitted signed payout
  PayoutReceived,
// both players signed payout
  PayoutCertified,
// payout tx broadcast
  PayoutTxBroadcast,
// payout tx in chain
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
  Certified,
// payout tx broadcast
  Broadcast,
// payout tx in chain
  Resolved,
// if only the arbiter has signed, invalid
  Invalid,
}

export interface JsonResponse {
    status: "success"|"error",
    data?: any,
    message?: string,
}
