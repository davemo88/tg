import { createStore } from 'redux';
import { nanoid, createEntityAdapter, createSlice, createReducer, createAction, configureStore, createAsyncThunk } from '@reduxjs/toolkit';
import { Secret, } from './secret';
import { EntityId, Player, Contract, Payout, } from './datatypes';

import PlayerWalletModule from './PlayerWallet';

const playerAdapter = createEntityAdapter<Player>({
    selectId: (player) => player.name,
});

export const playerSlice = createSlice({
    name: 'players',
    initialState: playerAdapter.getInitialState(),
    reducers: {
        playerAdded: playerAdapter.addOne,
        playerAddedMany: playerAdapter.addMany,
        playerRemoved: playerAdapter.removeOne,
    },
})

export const loadPlayers = () => {
    return async (dispatch) => {
        try {
            let output = await PlayerWalletModule.call_cli("player list --json-output");
            let players = JSON.parse(output);
            output = await PlayerWalletModule.call_cli("player mine --json-output");
            const my_players = JSON.parse(output);
            players.forEach(function (p) { 
                p.pictureUrl = "https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0";
                p.mine = my_players.some(mp => mp === p.name);
            });
            console.debug("loaded players:", players);
            return dispatch(playerSlice.actions.playerAddedMany(players));
        } catch (error) {
            console.log(error);
            return Promise.reject(error);
        }
    }
}

// might want to actually use createAsyncThunk for this one
export const newPlayer = (name: string, password: Secret<string>) => {
    return async (dispatch) => {
        try {
// confusing because of native module. what should its signature in typescript be?
            let cli_response: string = await PlayerWalletModule.call_cli_with_password(`player register "${name}"`, password.expose_secret()); 
            if (cli_response === "registered player") {
                return dispatch(playerSlice.actions.playerAdded({
                    id: name, 
                    name: name,
                    mine: true,
// TODO: set portrait based on player name hash
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
            if (cli_response === "added player") {
                return dispatch(playerSlice.actions.playerAdded({
                    id: name,
                    name: name,
                    mine: false,
// TODO: set portrait based on player name hash
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

export const removePlayer = (player: Player) => {
    return async (dispatch) => {
        try {
            let cli_response = await PlayerWalletModule.call_cli(`player remove "${player.name}"`);
            if (cli_response === "remove player") {
                return dispatch(playerSlice.actions.playerRemoved(player.name));
            } else {
                throw(cli_response);
            }
        } catch (error) {
            return Promise.reject(error);
        }
    }
}


const contractAdapter = createEntityAdapter<Contract>({
    selectId: (contract) => contract.cxid,
});

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
            console.debug("loaded contracts:", contracts);
            return dispatch(contractSlice.actions.contractAddedMany(contracts));
        } catch (error) {
            console.error(error);
            return Promise.reject(error);
        }
    }
}

export const newContract = (p1Name: string, p2Name: string, sats: number) => {
    return async (dispatch, getState) => {
        try {
            let p1 = playerSelectors.selectById(getState(), p1Name);
            let p2 = playerSelectors.selectById(getState(), p2Name);
            let cli_response = await PlayerWalletModule.call_cli(`contract new "${p1.name}" "${p2.name}" ${sats} --json-output`); 
            let contract_record = JSON.parse(cli_response);
            console.log("contract_record", contract_record);
            if (contract_record !== null) {
                return dispatch(contractSlice.actions.contractAdded({
                    cxid: contract_record.cxid,
                    playerOneName: p1Name,
                    playerTwoName: p2Name,
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

const payoutAdapter = createEntityAdapter<Payout>({
    selectId: (payout) => payout.cxid,
});

export const payoutSlice = createSlice({
  name: 'payouts',
  initialState: payoutAdapter.getInitialState(),
  reducers: {
    payoutAdded: payoutAdapter.addOne,
    payoutUpdated: payoutAdapter.updateOne,
    payoutRemoved: payoutAdapter.removeOne,
  }
})

// the selected player is the current player which the user is managing contracts for
export const selectedPlayerNameSlice = createSlice({
  name: 'selectedPlayerName',
  initialState: 'bogus selected player id',
  reducers: {
    setSelectedPlayerName:  (state, action) => action.payload
  }
})

export const getBalance = () => {
    return async (dispatch) => {
        try {
            let output = await PlayerWalletModule.call_cli("balance");
            let balance = +output;
            return dispatch(balanceSlice.actions.setBalance(balance));
        } catch (error) {
            return Promise.reject(error);
        }
    }
}

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
    payouts: payoutSlice.reducer,
    selectedPlayerName: selectedPlayerNameSlice.reducer,
    balance: balanceSlice.reducer,
  }
})

type RootState = ReturnType<typeof store.getState>

export const playerSelectors = playerAdapter.getSelectors<RootState>( state => state.players );
export const contractSelectors = contractAdapter.getSelectors<RootState>( state => state.contracts );
export const payoutSelectors = payoutAdapter.getSelectors<RootState>( state => state.payouts );

export const loadAll = () => {
    return dispatch =>
        Promise.all([
            dispatch(getBalance()),
            dispatch(loadPlayers()),
            dispatch(loadContracts()),
        ]);
}
