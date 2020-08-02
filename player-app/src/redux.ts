import { createStore } from 'redux';
import { nanoid, createEntityAdapter, createSlice, configureStore } from '@reduxjs/toolkit'
import { Player } from './datatypes.ts'
/*
  store data structure: 
  {
    playerIds: [1, 2, 3, ... ],
    players: {
      1: { player_1 },
      2: { player_2 },
      3: { player_3 },
            .
            .
            .
    },
    opponentIds: [1, 2, 3, ... ],
    opponents: {
      1: { opponent_1 },
      2: { opponent_2 },
      3: { opponent_3 },
            .
            .
            .
    },
    challengeIds: [1, 2, 3, ... ],
    challenges: {
      1: { challeneges_1 },
      2: { challeneges_2 },
      3: { challeneges_3 },
            .
            .
            .
    },
  } 
 */

const playerAdapter = createEntityAdapter<Player>({});

const playerSlice = createSlice({
  name: 'players',
  initialState: playerAdapter.getInitialState(),
  reducers: {
    playerAdded: playerAdapter.addOne,
  }
})

export const store = configureStore({
  reducer: {
    players: playerSlice.reducer,
  }
})

store.dispatch(playerSlice.actions.playerAdded({ id: nanoid(), name: 'Akin Toulouse', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0' }));
store.dispatch(playerSlice.actions.playerAdded({ id: nanoid(), name: 'Bacon Toulouse', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0' }));
store.dispatch(playerSlice.actions.playerAdded({ id: nanoid(), name: 'Curry Toulouse', pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0' }));

//type RootState = ReturnType<typeof store.getState>
//
//export const playerSelectors = playerAdapter.getSelectors<RootState>( state => state.players );
export const playerSelectors = playerAdapter.getSelectors( state => state.players );
