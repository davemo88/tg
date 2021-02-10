import { createStore } from 'redux';
import { nanoid, createEntityAdapter, createSlice, createReducer, createAction, configureStore, createAsyncThunk } from '@reduxjs/toolkit';
import { Player, Contract, PayoutRequest, } from './datatypes';

import PlayerWalletModule from './PlayerWallet';

const playerAdapter = createEntityAdapter<Player>({});

export const playerSlice = createSlice({
    name: 'players',
    initialState: playerAdapter.getInitialState(),
    reducers: {
        playerAdded: playerAdapter.addOne,
        playerAddedMany: playerAdapter.addMany,
    },
})

export const loadPlayers = () => {
    return async (dispatch) => {
        try {
            let output = await PlayerWalletModule.call_cli("player list --json-output");
            console.log("player list output:", output);
            let players = JSON.parse(output);
            console.log("player list:", players);
            output = await PlayerWalletModule.call_cli("player mine --json-output");
            console.log("player mine output:", output);
            const my_players = JSON.parse(output);
            console.log("my players:", my_players);
            players.forEach(function (p) { 
                p.id = nanoid(); 
                p.pictureUrl = "https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0";
                p.mine = my_players.some(mp => mp === p.name);
            });
            return dispatch(playerSlice.actions.playerAddedMany(players));
        } catch (error) {
            console.log(error);
            return Promise.reject(error);
        }
    }
}

// might want to actually use createAsyncThunk for this one
export const newPlayer = (name: string) => {
    return async (dispatch) => {
        try {
// confusing because of native module. what should its signature in typescript be?
            let cli_response: string = await PlayerWalletModule.call_cli(`player register "${name}"`); 
            console.log(cli_response);
            if (cli_response === "registered player") {
                return dispatch(playerSlice.actions.playerAdded({
                    id: nanoid(), 
                    name: name,
                    mine: true,
// yeah yeah i know 
                    pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0",
                }));
            } else {
                throw(cli_response);
            }
        } catch (error) {
            return Promise.reject(error);
        }
    }
}

export const addPlayer = (name: string) => {
    return async (dispatch) => {
        try {
            let cli_response = await PlayerWalletModule.call_cli(`player add "${name}"`);
            console.log(cli_response);
            if (cli_response === "added player") {
                return dispatch(playerSlice.actions.playerAdded({
                    id: nanoid(), 
                    name: name,
                    mine: false,
// yeah yeah i know 
                    pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0",
                }));
            } else {
                throw(cli_response);
            }
        } catch (error) {
            return Promise.reject(error);
        }
    }
}

const contractAdapter = createEntityAdapter<Contract>({});

export const contractSlice = createSlice({
  name: 'contracts',
  initialState: contractAdapter.getInitialState(),
  reducers: {
    contractAdded: contractAdapter.addOne,
    contractAddedMany: contractAdapter.addMany,
    contractUpdated: contractAdapter.updateOne,
    contractRemoved: contractAdapter.removeOne,
  }
})

export const loadContracts = () => {
    return async (dispatch) => {
        try {
            let cli_response = await PlayerWalletModule.call_cli("contract list --json-output");
            let contracts = JSON.parse(cli_response);
            contracts.forEach(function (c) {c.id = nanoid();});
            return dispatch(contractSlice.actions.contractAddedMany(contracts));
        } catch (error) {
            console.log(error);
            return Promise.reject(error);
        }
    }
}

export const newContract = (p1Id: string, p2Id: string, sats: number) => {
    return async (dispatch, getState) => {
        try {
            let p1 = playerSelectors.selectById(getState(), p1Id);
            let p2 = playerSelectors.selectById(getState(), p2Id);
            let cli_response = await PlayerWalletModule.call_cli(`contract new "${p1.name}" "${p2.name}" ${sats}`); 
            console.log("cli response", cli_response);
            if (cli_response === "contract created") {
                dispatch(balanceSlice.actions.setBalance(getState().balance - Math.ceil(sats/2)));
                return dispatch(contractSlice.actions.contractAdded({
                    id: nanoid(), 
                    playerOneId: p1Id,
                    playerTwoId: p2Id,
                    cxid: "",
                    amount: sats,
                    fundingTx: false,
                    playerOneSig: false,
                    playerTwoSig: false,
                    arbiterSig: false,
                }));
            } else {
                throw(cli_response);
            }
        } catch (error) {
            return Promise.reject(error);
        }
    }
}

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

export const loadAll = () => {
    return dispatch =>
        Promise.all([
            dispatch(loadPlayers()),
            dispatch(loadContracts()),
        ]);
}
