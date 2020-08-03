import { createStore } from 'redux';
import { nanoid, createEntityAdapter, createSlice, createReducer, createAction, configureStore } from '@reduxjs/toolkit'
import { Player, Opponent, Challenge, } from './datatypes.ts'

const playerAdapter = createEntityAdapter<Player>({});

export const playerSlice = createSlice({
  name: 'players',
  initialState: playerAdapter.getInitialState(),
  reducers: {
    playerAdded: playerAdapter.addOne,
  }
})

const opponentAdapter = createEntityAdapter<Opponent>({});

export const opponentSlice = createSlice({
  name: 'opponents',
  initialState: opponentAdapter.getInitialState(),
  reducers: {
    opponentAdded: opponentAdapter.addOne,
  }
})

const challengeAdapter = createEntityAdapter<Challenge>({});

export const challengeSlice = createSlice({
  name: 'challenges',
  initialState: challengeAdapter.getInitialState(),
  reducers: {
    challengeAdded: challengeAdapter.addOne,
  }
})

export const selectedPlayerIdSlice = createSlice({
  name: 'selectedPlayerId',
  initialState: 'bogus selected player id',
  reducers: {
    setSelectedPlayerId:  (state, action) => action.payload
  }
})

export const store = configureStore({
  reducer: {
    players: playerSlice.reducer,
    opponents: opponentSlice.reducer,
    challenges: challengeSlice.reducer,
    selectedPlayerId: selectedPlayerIdSlice.reducer,
  }
})

// test players
store.dispatch(playerSlice.actions.playerAdded({ id: nanoid(), name: 'Akin Toulouse', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0', balance: 9999 }));
store.dispatch(playerSlice.actions.playerAdded({ id: nanoid(), name: 'Duncan Hoops', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/111700/2.0', balance: 1111 }));
store.dispatch(playerSlice.actions.playerAdded({ id: nanoid(), name: 'Lesley Grillz', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/134255/2.0', balance: 2727 }));

// select first player
store.dispatch(
  selectedPlayerIdSlice.actions.setSelectedPlayerId(
    store.getState().players.ids[0]
  )
);

// test opponent
store.dispatch(opponentSlice.actions.opponentAdded({ id: nanoid(), name: 'Betsy Wildly', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0' }));

// test challenges
store.dispatch(challengeSlice.actions.challengeAdded({ 
  id: nanoid(),
  opponentId: store.getState().opponents.ids[0],
  pot: 256,
  status: 'Issued',
}))
store.dispatch(challengeSlice.actions.challengeAdded({ 
  id: nanoid(),
  opponentId: store.getState().opponents.ids[0],
  pot: 123,
  status: 'Certified',
}))

type RootState = ReturnType<typeof store.getState>

export const playerSelectors = playerAdapter.getSelectors<RootState>( state => state.players );
export const opponentSelectors = opponentAdapter.getSelectors<RootState>( state => state.opponents );
export const challengeSelectors = challengeAdapter.getSelectors<RootState>( state => state.challenges );
