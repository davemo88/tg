import { store, localPlayerSelectors, contractSelectors, payoutRequestSelectors } from './redux.ts';

import { LocalPlayer, Player, Contract, ContractStatus, PayoutRequest, PayoutRequestStatus, } from './datatypes.ts';


// TODO: this function checks the blockchain for the expected payout transactions for the contract
// if they aren't found then it isn't resolved. if they are invalid then the contract is invalid
// TODO: can only tell if contract is resolved by looking for txs that spend from fundingTx appropriately. suppose there are inappapropriate ones - then we refuse all payout requests?
// TODO: S is whatever the store type is, idk how to get it atm
export const getContractStatus = (contract: Contract ): ContractStatus => {
  const selectedPlayerId = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId).playerId;

  const allSigned: bool = (contract.playerOneSig && contract.playerTwoSig && contract.arbiterSig);

  if (allSigned && contract.fundingTx) {

    const payoutRequest = payoutRequestSelectors.selectAll(store.getState())
      .filter((pr, i, a) => pr.contractId === contract.id ).pop();

    if (payoutRequest) {
      const payoutRequestStatus = getPayoutRequestStatus(payoutRequest);
      switch (+payoutRequestStatus) {
        case PayoutRequestStatus.Unsigned:
          break;
        case PayoutRequestStatus.LocalPlayerSigned:
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
    return isContractSignedBy(contract, selectedPlayerId) ? ContractStatus.Issued : ContractStatus.Received;
  }
  else {
    return isUnsigned(contract) ? ContractStatus.Unsigned : ContractStatus.Invalid;
  }
}

export const getPayoutRequestStatus = (payoutRequest: PayoutRequest ): PayoutRequestStatus => {
  const selectedPlayerId = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId).playerId;
  const contract = contractSelectors.selectById(store.getState(), payoutRequest.contractId);
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
  else if (isPayoutRequestSignedBy(payoutRequest, selectedPlayerId)) {
    return PayoutRequestStatus.LocalPlayerSigned; 
  }
  else if (isPayoutRequestSignedBy(payoutRequest, getOtherPlayerId(selectedPlayerId, contract))) {
    return PayoutRequestStatus.OtherPlayerSigned; 
  }
  else {
    return isUnsigned(payoutRequest) ? PayoutRequestStatus.Unsigned : PayoutRequestStatus.Invalid;
  }
}

export const isContractSignedBy = (contract: Contract | PayoutRequest, playerId: PlayerId): bool => {
  return (
    ((contract.playerOneId === playerId) && contract.playerOneSig)
    ||
    ((contract.playerTwoId === playerId) && contract.playerTwoSig)
  )
}

export const isPayoutRequestSignedBy = (payoutRequest:  PayoutRequest, playerId: PlayerId): bool => {
  const contract = contractSelectors.selectById(store.getState(), payoutRequest.contractId);
  return (
    ((contract.playerOneId === playerId) && payoutRequest.playerOneSig)
    ||
    ((contract.playerTwoId === playerId) && payoutRequest.playerTwoSig)
  )
}

export const isUnsigned = (signable: Contract | PayoutRequest): bool => {
  return !(signable.playerOneSig || signable.playerTwoSig || signable.arbiterSig)
}

export const getOtherPlayerId = (playerId: PlayerId, contract: Contract): PlayerId | undefined => {
  if (contract.playerOneId === playerId) {
    return contract.playerTwoId;
  }
  else if (contract.playerTwoId === playerId) {
    return contract.playerOneId;
  }
}
