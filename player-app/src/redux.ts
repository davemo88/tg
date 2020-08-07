import { createStore } from 'redux';
import { nanoid, createEntityAdapter, createSlice, createReducer, createAction, configureStore } from '@reduxjs/toolkit'
import { Player, LocalPlayer, Opponent, Challenge, } from './datatypes.ts'

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

// the selected player is the current local player which the user is managing challenges for
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
    localPlayers: localPlayerSlice.reducer,
    opponents: opponentSlice.reducer,
    challenges: challengeSlice.reducer,
    selectedPlayerId: selectedPlayerIdSlice.reducer,
  }
})

// populate test store
// players
store.dispatch(playerSlice.actions.playerAdded({ id: 'akin', name: 'Akin Toulouse', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0', balance: 9999 }));
store.dispatch(playerSlice.actions.playerAdded({ id: 'duncan', name: 'Duncan Hoops', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/111700/2.0', balance: 1111 }));
store.dispatch(playerSlice.actions.playerAdded({ id: 'lesley', name: 'Lesley Grillz', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/134255/2.0', balance: 2727 }));
store.dispatch(playerSlice.actions.playerAdded({ id: 'betsy', name: 'Betsy Wildly', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0' }));
// local players
store.dispatch(localPlayerSlice.actions.localPlayerAdded({ id: 'local-akin', playerId: 'akin', balance: 9999 }))
store.dispatch(localPlayerSlice.actions.localPlayerAdded({ id: 'local-duncan', playerId: 'duncan', balance: 101, }))
store.dispatch(localPlayerSlice.actions.localPlayerAdded({ id: 'local-lesley', playerId: 'lesley', balance: 2727}))
// opponents
store.dispatch(opponentSlice.actions.opponentAdded({ id: 'opponent-betsy', playerId: 'betsy'}))


// select Akin
store.dispatch(
  selectedPlayerIdSlice.actions.setSelectedPlayerId(
    'akin'
  )
);

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
export const localPlayerSelectors = localPlayerAdapter.getSelectors<RootState>( state => state.localPlayers );
export const opponentSelectors = opponentAdapter.getSelectors<RootState>( state => state.opponents );
export const challengeSelectors = challengeAdapter.getSelectors<RootState>( state => state.challenges );
