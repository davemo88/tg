import { nanoid } from '@reduxjs/toolkit';
import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, payoutRequestSelectors, payoutRequestSlice, selectedPlayerIdSlice, } from './redux.ts';
import { Player, Url, Contract, PayoutRequest, } from './datatypes.ts';

// probably still s3 somewhere
export const STATIC_CONTENT_HOST: string = 'https://whatchadoinhere.s3.amazonaws.com/';
export const TITLE_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'cc.png'; 
export const TEST_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'test.png'; 
export const LIVE_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'live.png'; 

export const PASSPHRASE_MIN_LENGTH = 12;


// this is appdata
export const NETWORK: string = 'Test';

// delete some local data? set flag in db more likely
export const declineContract = (contractId: ContractId) => {
  store.dispatch(contractSlice.actions.contractRemoved(contractId));
}

export const dismissContract = (contractId: ContractId) => {
  store.dispatch(contractSlice.actions.contractRemoved(contractId));
}

export const denyPayoutRequest = (payoutRequestId: PayoutRequestId) => {
  store.dispatch(payoutRequestSlice.actions.payoutRequestRemoved(payoutRequestId));
}

// arbiter prefixed functions require calls to the arbiter service
export const arbiterSignContract = (contract: Contract) => {
// TODO: validation
  store.dispatch(contractSlice.actions.contractUpdated({
    id: contract.id,
    changes: { arbiterSig: true },
  }));
}

export const arbiterSignPayoutRequest = (payoutRequest: PayoutRequest) => {
  if (payoutRequest.payoutToken) {
    store.dispatch(payoutRequestSlice.actions.payoutRequestUpdated({
      id: payoutRequest.id,
      changes: { arbiterSig: true },
    }));
  }
}

//
// below functions need crypto wallet functions
//

export const newPlayer = (playerName: string, pictureUrl: Url) => {
  store.dispatch(playerSlice.actions.playerAdded({ id: nanoid(), name: playerName, pictureUrl: pictureUrl }));
}

export const createContract = (contract: Contract) => {
  store.dispatch
}

export const createPayoutRequest = (contract: Contract) => {
  store.dispatch
}

export const signContract = (contract: Contract) => {
  const selectedPlayerId = store.getState().selectedPlayerId;
  let action = {id: contract.id, changes: {}};
  if (contract.playerOneId === selectedPlayerId) {
    action.changes.playerOneSig = true;
  }
  else if (contract.playerTwoId === selectedPlayerId) {
    action.changes.playerTwoSig = true;
  }
  store.dispatch(contractSlice.actions.contractUpdated(action));
}

export const signPayoutRequest = (payoutRequest: PayoutRequest) => {
  const selectedPlayerId = store.getState().selectedPlayerId;
  const contract = contractSelectors.selectById(store.getState(), payoutRequest.contractId);
  let action = {id: payoutRequest.id, changes: {}};
  if (contract.playerOneId === selectedPlayerId) {
    action.changes.playerOneSig = true;
  }
  else if (contract.playerTwoId === selectedPlayerId) {
    action.changes.playerTwoSig = true;
  }
  store.dispatch(payoutRequestSlice.actions.payoutRequestUpdated(action));
}

export const broadcastFundingTx = (contract: Contract) => {
  store.dispatch(contractSlice.actions.contractUpdated({
    id: contract.id,
    changes: {
      fundingTx: true,
    }
  }));
}

export const broadcastPayoutTx = (payoutRequest: PayoutRequest) => {
  store.dispatch(payoutRequestSlice.actions.payoutRequestUpdated({
    id: payoutRequest.id,
    changes: {
      payoutTx: true,
    }
  }));

}
