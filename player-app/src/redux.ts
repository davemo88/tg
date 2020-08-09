import { createStore } from 'redux';
import { nanoid, createEntityAdapter, createSlice, createReducer, createAction, configureStore } from '@reduxjs/toolkit'
import { Player, LocalPlayer, Challenge, } from './datatypes.ts'

const playerAdapter = createEntityAdapter<Player>({});

export const playerSlice = createSlice({
  name: 'players',
  initialState: playerAdapter.getInitialState(),
  reducers: {
    playerAdded: playerAdapter.addOne,
  }
})

export const localPlayerAdapter = createEntityAdapter<LocalPlayer>({});

export const localPlayerSlice = createSlice({
  name: 'localPlayers',
  initialState: localPlayerAdapter.getInitialState(),
  reducers: {
    localPlayerAdded: localPlayerAdapter.addOne,
    localPlayerUpdated: localPlayerAdapter.updateOne,
  }
})

const challengeAdapter = createEntityAdapter<Challenge>({});

export const challengeSlice = createSlice({
  name: 'challenges',
  initialState: challengeAdapter.getInitialState(),
  reducers: {
    challengeAdded: challengeAdapter.addOne,
    challengeUpdated: challengeAdapter.updateOne,
  }
})

// the selected player is the current local player which the user is managing challenges for
export const selectedLocalPlayerIdSlice = createSlice({
  name: 'selectedLocalPlayerId',
  initialState: 'bogus selected player id',
  reducers: {
    setSelectedLocalPlayerId:  (state, action) => action.payload
  }
})

export const store = configureStore({
  reducer: {
    players: playerSlice.reducer,
    localPlayers: localPlayerSlice.reducer,
    challenges: challengeSlice.reducer,
    selectedLocalPlayerId: selectedLocalPlayerIdSlice.reducer,
  }
})

// populate test store
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

// select Akin by default
store.dispatch(
  selectedLocalPlayerIdSlice.actions.setSelectedLocalPlayerId(
    'local-akin'
  )
);

// test challenges
store.dispatch(challengeSlice.actions.challengeAdded({ 
  id: nanoid(),
  playerOneId: 'akin',
  playerTwoId: 'betsy',
  pot: 256,
  status: 'Live',
  funding_tx: true,
  playerOneSig: true,
  playerTwoSig: true,
  arbiterSig: true,
}))
store.dispatch(challengeSlice.actions.challengeAdded({ 
  id: nanoid(),
  playerOneId: 'akin',
  playerTwoId: 'stan',
  pot: 123,
  status: 'Certified',
  funding_tx: false,
  playerOneSig: true,
  playerTwoSig: true,
  arbiterSig: true,
}))
store.dispatch(challengeSlice.actions.challengeAdded({ 
  id: nanoid(),
  playerOneId: 'duncan',
  playerTwoId: 'betsy',
  pot: 6,
  status: 'Issued',
  funding_tx: false,
  playerOneSig: true,
  playerTwoSig: false,
  arbiterSig: false,
}))
store.dispatch(challengeSlice.actions.challengeAdded({ 
  id: nanoid(),
  playerOneId: 'lesley',
  playerTwoId: 'stan',
  pot: 11143,
  status: 'Live',
  funding_tx: true,
  playerOneSig: true,
  playerTwoSig: true,
  arbiterSig: true,
}))

type RootState = ReturnType<typeof store.getState>

export const playerSelectors = playerAdapter.getSelectors<RootState>( state => state.players );
export const localPlayerSelectors = localPlayerAdapter.getSelectors<RootState>( state => state.localPlayers );
export const challengeSelectors = challengeAdapter.getSelectors<RootState>( state => state.challenges );
