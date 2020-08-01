import { createEntityAdapter, createStore } from 'redux';
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
export const NEW_PLAYER = 'NEW_PLAYER';
export const SELECT_PLAYER = 'SELECT_PLAYER';

export function newplayer(player, playerId) {
  return {
    type: NEW_PLAYER,
    id: playerId,
    player,
  }
}

export function selectplayer(playerId) {
  return {
    type: SELECT_PLAYER,
    id: playerId,
  }
}

function playerReducer(state={ playerIds: [], players: {}, selectedPlayerId: null, }, action) {
  let newState = {};
  switch (action.type) {
    case NEW_PLAYER:
      newState = {
        playerIds: [ ...state.playerIds, action.id ],
        players: state.players,
        selectedPlayerId: state.selectedPlayer,
      };
      newState.players.set(action.id,action.player);
      return newState

    case SELECT_PLAYER:
      newState = {
        playerIds: state.playerIds,
        players: state.players,
        selectedPlayerId: action.id,
      };
      return newState


    default:
      return state
  }
} 

let initialState = {
  playerIds: [1,2,3],
  players: {
    1: {
      id: 1,
      name: 'Akin Toulouse',
      pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0',
    },
    2: {
      id: 2,
      name: 'Chad Dunkle',
      pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/111700/2.0',
    },
    3: {
      id: 3,
      name: 'Darby Raisins',
      pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/134254/2.0',
    },
  },
//  opponentIds: [],
//  opponents: {},
//  challengeIds: [],
//  challenges: {},
};
export const store = createStore(playerReducer, initialState, );
