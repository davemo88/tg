import { store, contractSelectors, payoutSelectors } from './redux';

import { Player, TxStatus, Contract, ContractStatus, Payout, PayoutStatus, } from './datatypes';


// TODO: this function checks the blockchain for the expected payout transactions for the contract
// if they aren't found then it isn't resolved. if they are invalid then the contract is invalid
// TODO: can only tell if contract is resolved by looking for txs that spend from fundingTx appropriately. suppose there are inappapropriate ones - then we refuse all payouts?
export const getContractStatus = (contract: Contract ): ContractStatus => {
    console.log(contract);
    const selectedPlayerName = store.getState().selectedPlayerName;

    const allSigned: boolean = (contract.p1Sig && contract.p2Sig && contract.arbiterSig);

    if (allSigned) {
      switch (+contract.txStatus) {
          case TxStatus.Unbroadcast:
              return ContractStatus.Certified
          case TxStatus.Broadcast:
              return ContractStatus.FundingTxBroadcast
          case TxStatus.Confirmed:
              break
      }

      const payout = payoutSelectors.selectAll(store.getState())
        .filter((pr, i, a) => pr.cxid === contract.cxid ).pop();

      if (payout) {
        const payoutStatus = getPayoutStatus(payout);
        switch (+payoutStatus) {
          case PayoutStatus.Unsigned:
            return ContractStatus.PayoutUnsigned
          case PayoutStatus.WeSigned:
            return ContractStatus.PayoutSigned
          case PayoutStatus.WeSignedWithToken:
            return ContractStatus.PayoutSignedWithToken
          case PayoutStatus.TheySigned:
            return ContractStatus.PayoutReceived
          case PayoutStatus.Certified:
            return ContractStatus.PayoutCertified
          case PayoutStatus.Broadcast:
            return ContractStatus.PayoutTxBroadcast
          case PayoutStatus.Resolved:
            return ContractStatus.Resolved
          case PayoutStatus.Invalid:
            return ContractStatus.Live
        }
      }
      else {
        return ContractStatus.Live
      }
    }
    else if (contract.p1Sig && contract.p2Sig) {
      return ContractStatus.PlayersSigned
    }
    else if ((contract.p1Sig || contract.p2Sig) && !contract.arbiterSig) {
      return isContractSignedBy(contract, selectedPlayerName) ? ContractStatus.Signed: ContractStatus.Received
    }
    else {
      return isUnsigned(contract) ? ContractStatus.Unsigned : ContractStatus.Invalid;
    }
    return ContractStatus.Invalid
}

export const getPayoutStatus = (payout: Payout ): PayoutStatus => {
    const selectedPlayerName = store.getState().selectedPlayerName;
    const contract = contractSelectors.selectById(store.getState(), payout.cxid);

    switch (+payout.txStatus) {
        case TxStatus.Unbroadcast:
            break;
        case TxStatus.Broadcast:
            return PayoutStatus.Broadcast
        case TxStatus.Confirmed:
            return PayoutStatus.Resolved

    }

    if (
      ((payout.p1Sig && payout.p2Sig))
      || 
      ((payout.p1Sig || payout.p2Sig) && payout.arbiterSig) 
    ) {
      return PayoutStatus.Certified;
    }
    else if (isPayoutSignedBy(payout, selectedPlayerName)) {
      if (payout.scriptSig) {
        return PayoutStatus.WeSignedWithToken; 
      } else {
        return PayoutStatus.WeSigned; 
      }
    }
    else if (contract && isPayoutSignedBy(payout, getOtherPlayerName(selectedPlayerName, contract))) {
      return PayoutStatus.TheySigned; 
    }
    else {
      return isUnsigned(payout) ? PayoutStatus.Unsigned : PayoutStatus.Invalid;
    }
}

export const isContractSignedBy = (contract: Contract, playerName: string): boolean => {
  return (
    ((contract.p1Name === playerName) && contract.p1Sig)
    ||
    ((contract.p2Name === playerName) && contract.p2Sig)
  )
}

export const isPayoutSignedBy = (payout:  Payout, playerName?: string): boolean => {
  const contract = contractSelectors.selectById(store.getState(), payout.cxid);
  if (!contract) {
      console.error(Error("payout doesn't have a corresponding contract"));
      return false
  }
  return (
    ((contract.p1Name === playerName) && payout.p1Sig)
    ||
    ((contract.p2Name === playerName) && payout.p2Sig)
  )
}

export const isUnsigned = (signable: Contract | Payout): boolean => {
  return !(signable.p1Sig || signable.p2Sig || signable.arbiterSig)
}

export const getOtherPlayerName = (playerName: string, contract: Contract):  string | undefined => {
  if (contract.p1Name === playerName) {
    return contract.p2Name;
  }
  else if (contract.p2Name === playerName) {
    return contract.p1Name;
  }
}
