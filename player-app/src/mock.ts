import { nanoid } from '@reduxjs/toolkit';
import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, selectedLocalPlayerIdSlice, } from './redux.ts';

export const populateTestStore = () => {
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
    id: nanoid(),
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
  }))
  
}
