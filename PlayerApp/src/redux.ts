import { createStore } from 'redux';
import { createEntityAdapter, createSlice, createReducer, createAction, configureStore, createAsyncThunk } from '@reduxjs/toolkit';
import { Secret, } from './secret';
import { JsonResponse, Status, Player, Contract, Payout, } from './datatypes';

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
            let response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli("player list"));
            if (response.status === "error") {
                throw(response.message);
            }
            let players: [Player] = response.data;
            response = JSON.parse(await PlayerWalletModule.call_cli("player mine"));
            if (response.status === "error") {
                throw(response.message);
            }
            const my_players: [string] = response.data;
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
    return async (dispatch, getState) => {
        try {
// confusing because of native module. what should its signature in typescript be?
            let response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli_with_password(`player register "${name}"`, password.expose_secret())); 
            if (response.status === "error") {
                throw(response.message);
            }
            return dispatch(playerSlice.actions.playerAdded({
                name: name,
                mine: true,
// TODO: set portrait based on player name hash
                pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0",
            }));
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
            let response = JSON.parse(await PlayerWalletModule.call_cli("contract list"));
            if (response.status === "error") {
                throw(response.message);
            }
            let contracts = response.data;
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
            let cli_response = await PlayerWalletModule.call_cli(`contract new "${p1.name}" "${p2.name}" ${sats}`); 
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
  initialState: 'bogus selected player name',
  reducers: {
    setSelectedPlayerName:  (state, action) => action.payload
  }
})

export const balanceSlice = createSlice({
  name: 'balance',
  initialState: 0,
  reducers: {
    setBalance:  (state, action) => action.payload
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

export const postedSlice = createSlice({
  name: 'posted',
  initialState: 0,
  reducers: {
    setPosted:  (state, action) => action.payload
  }
})

export const getPosted = (name: string) => {
    return async (dispatch) => {
        try {
            const cli_output = await PlayerWalletModule.call_cli(`player posted "${name}"`);
            let response: JsonResponse = JSON.parse(cli_output);
            let posted = +response.data
            console.debug("posted:", posted);
            return dispatch(postedSlice.actions.setPosted(posted));
        } catch (error) {
            return Promise.reject(error);
        }
    }
}

export const store = configureStore({
  reducer: {
    players: playerSlice.reducer,
    contracts: contractSlice.reducer,
    payouts: payoutSlice.reducer,
    selectedPlayerName: selectedPlayerNameSlice.reducer,
    balance: balanceSlice.reducer,
    posted: postedSlice.reducer,
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
