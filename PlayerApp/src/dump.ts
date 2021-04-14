import { store, contractSelectors, payoutSelectors } from './redux';

import { Player, Contract, ContractStatus, Payout, PayoutStatus, } from './datatypes';


// TODO: this function checks the blockchain for the expected payout transactions for the contract
// if they aren't found then it isn't resolved. if they are invalid then the contract is invalid
// TODO: can only tell if contract is resolved by looking for txs that spend from fundingTx appropriately. suppose there are inappapropriate ones - then we refuse all payouts?
// TODO: S is whatever the store type is, idk how to get it atm
export const getContractStatus = (contract: Contract ): ContractStatus => {
  const selectedPlayerName = store.getState().selectedPlayerName;

  const allSigned: bool = (contract.playerOneSig && contract.playerTwoSig && contract.arbiterSig);

  if (allSigned && contract.fundingTx) {

    const payout = payoutSelectors.selectAll(store.getState())
      .filter((pr, i, a) => pr.cxid === contract.cxid ).pop();

    if (payout) {
      const payoutStatus = getPayoutStatus(payout);
      switch (+payoutStatus) {
        case PayoutStatus.Unsigned:
          break;
        case PayoutStatus.SelectedPlayerSigned:
          return ContractStatus.PayoutIssued;
        case PayoutStatus.OtherPlayerSigned:
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
  else if (contract.playerOneSig && contract.playerTwoSig) {
    return ContractStatus.Accepted;
  }
  else if ((contract.playerOneSig || contract.playerTwoSig) && !contract.arbiterSig) {
    return isContractSignedBy(contract, selectedPlayerName) ? ContractStatus.Issued : ContractStatus.Received;
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
    ((payout.playerOneSig && payout.playerTwoSig))
    || 
    ((payout.playerOneSig || payout.playerTwoSig) && payout.arbiterSig) 
  ) {
    return PayoutStatus.Live;
  }
  else if (isPayoutSignedBy(payout, selectedPlayerName)) {
    return PayoutStatus.SelectedPlayerSigned; 
  }
  else if (isPayoutSignedBy(payout, getOtherPlayerName(selectedPlayerName, contract))) {
    return PayoutStatus.OtherPlayerSigned; 
  }
  else {
    return isUnsigned(payout) ? PayoutStatus.Unsigned : PayoutStatus.Invalid;
  }
}

export const isContractSignedBy = (contract: Contract | Payout, playerName: PlayerName): bool => {
  return (
    ((contract.playerOneName === playerName) && contract.playerOneSig)
    ||
    ((contract.playerTwoName === playerName) && contract.playerTwoSig)
  )
}

export const isPayoutSignedBy = (payout:  Payout, playerName: PlayerName): bool => {
  const contract = contractSelectors.selectById(store.getState(), payout.cxid);
  return (
    ((contract.playerOneName === playerName) && payout.playerOneSig)
    ||
    ((contract.playerTwoName === playerName) && payout.playerTwoSig)
  )
}

export const isUnsigned = (signable: Contract | Payout): bool => {
  return !(signable.playerOneSig || signable.playerTwoSig || signable.arbiterSig)
}

export const getOtherPlayerName = (playerName: PlayerName, contract: Contract): PlayerName | undefined => {
  if (contract.playerOneName === playerName) {
    return contract.playerTwoName;
  }
  else if (contract.playerTwoName === playerName) {
    return contract.playerOneName;
  }
}
