import { nanoid } from '@reduxjs/toolkit';
import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, contractSelectors, contractSlice, payoutRequestSelectors, payoutRequestSlice, selectedLocalPlayerIdSlice, } from './redux.ts';
import { LocalPlayer, Player, Url, Contract, PayoutRequest, } from './datatypes.ts';

// probably still s3 somewhere
export const STATIC_CONTENT_HOST: string = 'https://whatchadoinhere.s3.amazonaws.com/';
export const TITLE_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'cc.png'; 
export const TEST_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'test.png'; 
export const LIVE_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'live.png'; 


// this is appdata
export const NETWORK: string = 'Test';
export const loadLocalData = () => {
  // players
  store.dispatch(playerSlice.actions.playerAdded({ id: 'akin', name: 'Akin Toulouse', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0', balance: 9999 }));
  store.dispatch(playerSlice.actions.playerAdded({ id: 'duncan', name: 'Duncan Hoops', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/111700/2.0', balance: 1111 }));
  store.dispatch(playerSlice.actions.playerAdded({ id: 'lesley', name: 'Lesley', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/134255/2.0', balance: 2727 }));
  store.dispatch(playerSlice.actions.playerAdded({ id: 'betsy', name: 'Betsy Wildly', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0' }));
  store.dispatch(playerSlice.actions.playerAdded({ id: 'stan', name: 'Stan Dandyliver', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/68856/2.0' }));
  // local players
  store.dispatch(localPlayerSlice.actions.localPlayerAdded({ id: 'local-akin', playerId: 'akin', balance: 9999 }));
  store.dispatch(localPlayerSlice.actions.localPlayerAdded({ id: 'local-duncan', playerId: 'duncan', balance: 101, }));
  store.dispatch(localPlayerSlice.actions.localPlayerAdded({ id: 'local-lesley', playerId: 'lesley', balance: 2727}));
  
  // test contracts
  store.dispatch(contractSlice.actions.contractAdded({ 
    id: 'akin-v-betsy',
    playerOneId: 'akin',
    playerTwoId: 'betsy',
    pot: 256,
    fundingTx: true,
    playerOneSig: true,
    playerTwoSig: true,
    arbiterSig: true,
  }));
  store.dispatch(contractSlice.actions.contractAdded({ 
    id: 'akin-v-betsy-invalid',
    playerOneId: 'akin',
    playerTwoId: 'betsy',
    pot: 69,
    fundingTx: true,
    playerOneSig: false,
    playerTwoSig: false,
    arbiterSig: true,
  }));
  store.dispatch(contractSlice.actions.contractAdded({ 
    id: nanoid(),
    playerOneId: 'akin',
    playerTwoId: 'stan',
    pot: 123,
    fundingTx: false,
    playerOneSig: true,
    playerTwoSig: true,
    arbiterSig: true,
  }));
  store.dispatch(contractSlice.actions.contractAdded({ 
    id: nanoid(),
    playerOneId: 'lesley',
    playerTwoId: 'akin',
    pot: 6,
    fundingTx: false,
    playerOneSig: true,
    playerTwoSig: false,
    arbiterSig: false,
  }));
  store.dispatch(contractSlice.actions.contractAdded({ 
    id: nanoid(),
    playerOneId: 'duncan',
    playerTwoId: 'betsy',
    pot: 6,
    fundingTx: false,
    playerOneSig: true,
    playerTwoSig: false,
    arbiterSig: false,
  }));
  store.dispatch(contractSlice.actions.contractAdded({ 
    id: nanoid(),
    playerOneId: 'lesley',
    playerTwoId: 'stan',
    pot: 11143,
    fundingTx: true,
    playerOneSig: true,
    playerTwoSig: true,
    arbiterSig: true,
  }));
// payout requests
  store.dispatch(payoutRequestSlice.actions.payoutRequestAdded({
    id: nanoid(),
    contractId: 'akin-v-betsy',
    payoutTx: false,
    playerOneSig: false,
    playerTwoSig: true,
    arbiterSig: false,
    playerOneAmount: 0,
    playerTwoAmount: 256,
  }));
}

// delete some local data? set local flag more likely
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

export const newLocalPlayer = (playerName: string, pictureUrl: Url) => {
  const newPlayerId = nanoid();
  store.dispatch(playerSlice.actions.playerAdded({ id: newPlayerId, name: playerName, pictureUrl: pictureUrl }));
  store.dispatch(localPlayerSlice.actions.localPlayerAdded({ id: nanoid(), playerId: newPlayerId, balance: 0 }));
}

export const createContract = (contract: Contract) => {
  store.dispatch
}

export const createPayoutRequest = (contract: Contract) => {
  store.dispatch
}

export const signContract = (contract: Contract) => {
  const selectedPlayerId = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId).playerId;
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
  const selectedPlayerId = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId).playerId;
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
