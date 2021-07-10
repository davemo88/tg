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
    p1TokenDesc:        string,
    p2TokenDesc:        string,
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
// payout signed by us
  PayoutSigned,
// payout signed by us and token was included
  PayoutSignedWithToken,
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
  WeSignedWithToken,
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

export type Event = {
    desc: string,
    oracle_pubkey: string,
    outcomes: Outcome[],
}

export type Outcome = {
    desc: string,
    token: string,
    sig?: string,
}

const isOutcome = (outcome: any): boolean => {
    return outcome
        && outcome.desc && typeof(outcome.desc) == 'string'
        && outcome.token && typeof(outcome.token) == 'string'
}

export const isEvent = (event: any): boolean => {
    return event 
        && event.desc && typeof(event.desc) == 'string'
        && event.oracle_pubkey && typeof(event.oracle_pubkey) == 'string'
        && event.outcomes && Array.isArray(event.outcomes) 
        && event.outcomes.every((outcome: any) => isOutcome(outcome))
}
