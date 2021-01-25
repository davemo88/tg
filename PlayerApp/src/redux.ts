import { createStore } from 'redux';
import { nanoid, createEntityAdapter, createSlice, createReducer, createAction, configureStore } from '@reduxjs/toolkit'
import { Player, Contract, PayoutRequest, } from './datatypes.ts'

const playerAdapter = createEntityAdapter<Player>({});

export const playerSlice = createSlice({
  name: 'players',
  initialState: playerAdapter.getInitialState(),
  reducers: {
    playerAdded: playerAdapter.addOne,
  }
})

const contractAdapter = createEntityAdapter<Contract>({});

export const contractSlice = createSlice({
  name: 'contracts',
  initialState: contractAdapter.getInitialState(),
  reducers: {
    contractAdded: contractAdapter.addOne,
    contractUpdated: contractAdapter.updateOne,
    contractRemoved: contractAdapter.removeOne,
  }
})

const payoutRequestAdapter = createEntityAdapter<PayoutRequest>({});

export const payoutRequestSlice = createSlice({
  name: 'payoutRequests',
  initialState: payoutRequestAdapter.getInitialState(),
  reducers: {
    payoutRequestAdded: payoutRequestAdapter.addOne,
    payoutRequestUpdated: payoutRequestAdapter.updateOne,
    payoutRequestRemoved: payoutRequestAdapter.removeOne,
  }
})

// the selected player is the current player which the user is managing contracts for
export const selectedPlayerIdSlice = createSlice({
  name: 'selectedPlayerId',
  initialState: 'bogus selected player id',
  reducers: {
    setSelectedPlayerId:  (state, action) => action.payload
  }
})

export const balanceSlice = createSlice({
  name: 'balance',
  initialState: 0,
  reducers: {
    setBalance:  (state, action) => action.payload
  }
})

export const store = configureStore({
  reducer: {
    players: playerSlice.reducer,
    contracts: contractSlice.reducer,
    payoutRequests: payoutRequestSlice.reducer,
    selectedPlayerId: selectedPlayerIdSlice.reducer,
    balance: balanceSlice.reducer,
  }
})

type RootState = ReturnType<typeof store.getState>

export const playerSelectors = playerAdapter.getSelectors<RootState>( state => state.players );
export const contractSelectors = contractAdapter.getSelectors<RootState>( state => state.contracts );
export const payoutRequestSelectors = payoutRequestAdapter.getSelectors<RootState>( state => state.payoutRequests );
