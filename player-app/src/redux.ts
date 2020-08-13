import { createStore } from 'redux';
import { nanoid, createEntityAdapter, createSlice, createReducer, createAction, configureStore } from '@reduxjs/toolkit'
import { Player, LocalPlayer, Challenge, PayoutRequest, } from './datatypes.ts'

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

const payoutRequestAdapter = createEntityAdapter<PayoutRequest>({});

export const payoutRequestSlice = createSlice({
  name: 'payoutRequests',
  initialState: payoutRequestAdapter.getInitialState(),
  reducers: {
    payoutRequestAdded: payoutRequestAdapter.addOne,
    payoutRequestUpdated: payoutRequestAdapter.updateOne,
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
    payoutRequests: payoutRequestSlice.reducer,
    selectedLocalPlayerId: selectedLocalPlayerIdSlice.reducer,
  }
})

type RootState = ReturnType<typeof store.getState>

export const playerSelectors = playerAdapter.getSelectors<RootState>( state => state.players );
export const localPlayerSelectors = localPlayerAdapter.getSelectors<RootState>( state => state.localPlayers );
export const challengeSelectors = challengeAdapter.getSelectors<RootState>( state => state.challenges );
export const payoutRequestSelectors = payoutRequestAdapter.getSelectors<RootState>( state => state.payoutRequests );
