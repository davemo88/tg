import { nanoid } from '@reduxjs/toolkit';
import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, payoutRequestSelectors, payoutRequestSlice, selectedLocalPlayerIdSlice, } from './redux.ts';

export const loadLocalData = () => {
  // players
  store.dispatch(playerSlice.actions.playerAdded({ id: 'akin', name: 'Akin Toulouse', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0', balance: 9999 }));
  store.dispatch(playerSlice.actions.playerAdded({ id: 'duncan', name: 'Duncan Hoops', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/111700/2.0', balance: 1111 }));
  store.dispatch(playerSlice.actions.playerAdded({ id: 'lesley', name: 'Lesley', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/134255/2.0', balance: 2727 }));
  store.dispatch(playerSlice.actions.playerAdded({ id: 'betsy', name: 'Betsy Wildly', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0' }));
  store.dispatch(playerSlice.actions.playerAdded({ id: 'stan', name: 'Stan Dandyliver', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/68856/2.0' }));
  // local players
  store.dispatch(localPlayerSlice.actions.localPlayerAdded({ id: 'local-akin', playerId: 'akin', balance: 9999 }))
  store.dispatch(localPlayerSlice.actions.localPlayerAdded({ id: 'local-duncan', playerId: 'duncan', balance: 101, }))
  store.dispatch(localPlayerSlice.actions.localPlayerAdded({ id: 'local-lesley', playerId: 'lesley', balance: 2727}))
  
  // test challenges
  store.dispatch(challengeSlice.actions.challengeAdded({ 
    id: 'akin-v-betsy',
    playerOneId: 'akin',
    playerTwoId: 'betsy',
    pot: 256,
    fundingTx: true,
    playerOneSig: true,
    playerTwoSig: true,
    arbiterSig: true,
  }))
  store.dispatch(challengeSlice.actions.challengeAdded({ 
    id: nanoid(),
    playerOneId: 'akin',
    playerTwoId: 'stan',
    pot: 123,
    fundingTx: false,
    playerOneSig: true,
    playerTwoSig: true,
    arbiterSig: true,
  }))
  store.dispatch(challengeSlice.actions.challengeAdded({ 
    id: nanoid(),
    playerOneId: 'lesley',
    playerTwoId: 'akin',
    pot: 6,
    fundingTx: false,
    playerOneSig: true,
    playerTwoSig: false,
    arbiterSig: false,
  }))
  store.dispatch(challengeSlice.actions.challengeAdded({ 
    id: nanoid(),
    playerOneId: 'duncan',
    playerTwoId: 'betsy',
    pot: 6,
    fundingTx: false,
    playerOneSig: true,
    playerTwoSig: false,
    arbiterSig: false,
  }))
  store.dispatch(challengeSlice.actions.challengeAdded({ 
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
    challengeId: 'akin-v-betsy',
    payoutTx: false,
    playerOneSig: false,
    playerTwoSig: true,
    arbiterSig: false,
    playerOneAmount: 0,
    playerTwoAmount: 256,
  }));
}

// this is 
export const createChallenge = (challenge: Challenge) => {
  store.dispatch
}

export const createPayoutRequest = (challenge: Challenge) => {
  store.dispatch
}

export const signChallenge = (challenge: Challenge) => {
  const selectedPlayerId = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId).playerId;
  let action = {id: challenge.id, changes: {}};
  if (challenge.playerOneId === selectedPlayerId) {
    action.changes.playerOneSig = true;
  }
  else if (challenge.playerTwoId === selectedPlayerId) {
    action.changes.playerTwoSig = true;
  }
  store.dispatch(challengeSlice.actions.challengeUpdated(action));
}

export const signPayoutRequest = (payoutRequest: PayoutRequest) => {
  const selectedPlayerId = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId).playerId;
  const challenge = challengeSelectors.selectById(store.getState(), payoutRequest.challengeId);
  let action = {id: payoutRequest.id, changes: {}};
  if (challenge.playerOneId === selectedPlayerId) {
    action.changes.playerOneSig = true;
  }
  else if (challenge.playerTwoId === selectedPlayerId) {
    action.changes.playerTwoSig = true;
  }
  store.dispatch(payoutRequestSlice.actions.payoutRequestUpdated(action));
}

export const broadcastFundingTx = (challenge: Challenge) => {
  store.dispatch(challengeSlice.actions.challengeUpdated({
    id: challenge.id,
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

// these arbiter prefixed functions require calls to the arbiter service to resolve
export const arbiterSignChallenge = (challenge: Challenge) => {
// TODO: validation
  store.dispatch(challengeSlice.actions.challengeUpdated({
    id: challenge.id,
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

export const declineChallenge = (challengeId: ChallengeId) => {
  store.dispatch(challengeSlice.actions.challengeRemoved(challengeId));
}

export const denyPayoutRequest = (payoutRequestId: PayoutRequestId) => {
  store.dispatch(payoutRequestSlice.actions.payoutRequestRemoved(payoutRequestId));
}
