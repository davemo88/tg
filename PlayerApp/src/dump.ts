import { store, contractSelectors, payoutSelectors } from './redux';

import { Player, Contract, ContractStatus, Payout, PayoutStatus, } from './datatypes';


// TODO: this function checks the blockchain for the expected payout transactions for the contract
// if they aren't found then it isn't resolved. if they are invalid then the contract is invalid
// TODO: can only tell if contract is resolved by looking for txs that spend from fundingTx appropriately. suppose there are inappapropriate ones - then we refuse all payouts?
// TODO: S is whatever the store type is, idk how to get it atm
export const getContractStatus = (contract: Contract ): ContractStatus => {
  const selectedPlayerName = store.getState().selectedPlayerName;

  const allSigned: bool = (contract.p1Sig && contract.p2Sig && contract.arbiterSig);

  if (allSigned && contract.fundingTx) {

    const payout = payoutSelectors.selectAll(store.getState())
      .filter((pr, i, a) => pr.cxid === contract.cxid ).pop();

    if (payout) {
      const payoutStatus = getPayoutStatus(payout);
      switch (+payoutStatus) {
        case PayoutStatus.Unsigned:
          return ContractStatus.PayoutUnsigned;
        case PayoutStatus.WeSigned:
          return ContractStatus.PayoutSigned;
        case PayoutStatus.TheySigned:
          return ContractStatus.PayoutReceived;
        case PayoutStatus.Live:
          return ContractStatus.PayoutLive;
        case PayoutStatus.Resolved:
          return ContractStatus.Resolved;
        case PayoutStatus.Invalid:
          return ContractStatus.Live;
      }
    }
    else {
      return ContractStatus.Live;
    }
  }
  else if (allSigned) {
    return ContractStatus.Certified;
  }
  else if (contract.p1Sig && contract.p2Sig) {
    return ContractStatus.PlayersSigned;
  }
  else if ((contract.p1Sig || contract.p2Sig) && !contract.arbiterSig) {
    return isContractSignedBy(contract, selectedPlayerName) ? ContractStatus.Signed: ContractStatus.Received;
  }
  else {
    return isUnsigned(contract) ? ContractStatus.Unsigned : ContractStatus.Invalid;
  }
}

export const getPayoutStatus = (payout: Payout ): PayoutStatus => {
  const selectedPlayerName = store.getState().selectedPlayerName;
  const contract = contractSelectors.selectById(store.getState(), payout.cxid);
  if (payout.payoutTx) {
    return PayoutStatus.Resolved;
  }
  else if (
    ((payout.p1Sig && payout.p2Sig))
    || 
    ((payout.p1Sig || payout.p2Sig) && payout.arbiterSig) 
  ) {
    return PayoutStatus.Live;
  }
  else if (isPayoutSignedBy(payout, selectedPlayerName)) {
    return PayoutStatus.WeSigned; 
  }
  else if (isPayoutSignedBy(payout, getOtherPlayerName(selectedPlayerName, contract))) {
    return PayoutStatus.TheySigned; 
  }
  else {
    return isUnsigned(payout) ? PayoutStatus.Unsigned : PayoutStatus.Invalid;
  }
}

export const isContractSignedBy = (contract: Contract | Payout, playerName: PlayerName): bool => {
  return (
    ((contract.p1Name === playerName) && contract.p1Sig)
    ||
    ((contract.p2Name === playerName) && contract.p2Sig)
  )
}

export const isPayoutSignedBy = (payout:  Payout, playerName: PlayerName): bool => {
  const contract = contractSelectors.selectById(store.getState(), payout.cxid);
  return (
    ((contract.p1Name === playerName) && payout.p1Sig)
    ||
    ((contract.p2Name === playerName) && payout.p2Sig)
  )
}

export const isUnsigned = (signable: Contract | Payout): bool => {
  return !(signable.p1Sig || signable.p2Sig || signable.arbiterSig)
}

export const getOtherPlayerName = (playerName: PlayerName, contract: Contract): PlayerName | undefined => {
  if (contract.p1Name === playerName) {
    return contract.p2Name;
  }
  else if (contract.p2Name === playerName) {
    return contract.p1Name;
  }
}
