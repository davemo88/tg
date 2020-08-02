import { createStore } from 'redux';
import { nanoid, createEntityAdapter, createSlice, createReducer, createAction, configureStore } from '@reduxjs/toolkit'
import { Player } from './datatypes.ts'

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
    selectedPlayerId: selectedPlayerIdSlice.reducer,
  }
})

// test players
store.dispatch(playerSlice.actions.playerAdded({ id: nanoid(), name: 'Akin Toulouse', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0' }));
store.dispatch(playerSlice.actions.playerAdded({ id: nanoid(), name: 'Duncan Hoops', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/111700/2.0' }));
store.dispatch(playerSlice.actions.playerAdded({ id: nanoid(), name: 'Lesley Grillz', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/134255/2.0' }));

// select first player
store.dispatch(selectedPlayerIdSlice.actions.setSelectedPlayerId(store.getState().players.ids[0]));

// test opponent
store.dispatch(opponentSlice.actions.opponentAdded({ id: nanoid(), name: 'Betsy Wildly', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/03259/2.0' }));

type RootState = ReturnType<typeof store.getState>

export const playerSelectors = playerAdapter.getSelectors<RootState>( state => state.players );
export const opponentSelectors = opponentAdapter.getSelectors<RootState>( state => state.opponents );
