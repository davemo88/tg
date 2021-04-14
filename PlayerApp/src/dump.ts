import { store, contractSelectors, payoutRequestSelectors } from './redux';

import { Player, Contract, ContractStatus, PayoutRequest, PayoutRequestStatus, } from './datatypes';


// TODO: this function checks the blockchain for the expected payout transactions for the contract
// if they aren't found then it isn't resolved. if they are invalid then the contract is invalid
// TODO: can only tell if contract is resolved by looking for txs that spend from fundingTx appropriately. suppose there are inappapropriate ones - then we refuse all payout requests?
// TODO: S is whatever the store type is, idk how to get it atm
export const getContractStatus = (contract: Contract ): ContractStatus => {
  const selectedPlayerName = store.getState().selectedPlayerName;

  const allSigned: bool = (contract.playerOneSig && contract.playerTwoSig && contract.arbiterSig);

  if (allSigned && contract.fundingTx) {

    const payoutRequest = payoutRequestSelectors.selectAll(store.getState())
      .filter((pr, i, a) => pr.cxid === contract.cxid ).pop();

    if (payoutRequest) {
      const payoutRequestStatus = getPayoutRequestStatus(payoutRequest);
      switch (+payoutRequestStatus) {
        case PayoutRequestStatus.Unsigned:
          break;
        case PayoutRequestStatus.SelectedPlayerSigned:
          return ContractStatus.PayoutRequestIssued;
        case PayoutRequestStatus.OtherPlayerSigned:
          return ContractStatus.PayoutRequestReceived;
        case PayoutRequestStatus.Live:
          return ContractStatus.PayoutRequestLive;
        case PayoutRequestStatus.Resolved:
          return ContractStatus.Resolved;
        case PayoutRequestStatus.Invalid:
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

export const getPayoutRequestStatus = (payoutRequest: PayoutRequest ): PayoutRequestStatus => {
  const selectedPlayerName = store.getState().selectedPlayerName;
  const contract = contractSelectors.selectById(store.getState(), payout.cxid);
  if (payoutRequest.payoutTx) {
    return PayoutRequestStatus.Resolved;
  }
  else if (
    ((payoutRequest.playerOneSig && payoutRequest.playerTwoSig))
    || 
    ((payoutRequest.playerOneSig || payoutRequest.playerTwoSig) && payoutRequest.arbiterSig) 
  ) {
    return PayoutRequestStatus.Live;
  }
  else if (isPayoutRequestSignedBy(payoutRequest, selectedPlayerName)) {
    return PayoutRequestStatus.SelectedPlayerSigned; 
  }
  else if (isPayoutRequestSignedBy(payoutRequest, getOtherPlayerName(selectedPlayerName, contract))) {
    return PayoutRequestStatus.OtherPlayerSigned; 
  }
  else {
    return isUnsigned(payoutRequest) ? PayoutRequestStatus.Unsigned : PayoutRequestStatus.Invalid;
  }
}

export const isContractSignedBy = (contract: Contract | PayoutRequest, playerName: PlayerName): bool => {
  return (
    ((contract.playerOneName === playerName) && contract.playerOneSig)
    ||
    ((contract.playerTwoName === playerName) && contract.playerTwoSig)
  )
}

export const isPayoutRequestSignedBy = (payoutRequest:  PayoutRequest, playerName: PlayerName): bool => {
  const contract = contractSelectors.selectById(store.getState(), payout.cxid);
  return (
    ((contract.playerOneName === playerName) && payoutRequest.playerOneSig)
    ||
    ((contract.playerTwoName === playerName) && payoutRequest.playerTwoSig)
  )
}

export const isUnsigned = (signable: Contract | PayoutRequest): bool => {
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
