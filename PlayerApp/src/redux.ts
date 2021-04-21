import { createStore } from 'redux';
import { createEntityAdapter, createSlice, createReducer, createAction, configureStore, createAsyncThunk } from '@reduxjs/toolkit';
import { Secret, } from './secret';
import { JsonResponse, Player, Contract, Payout, } from './datatypes';

import { getPosted } from './wallet';
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
        playerUpdated: playerAdapter.updateOne,
        playerRemoved: playerAdapter.removeOne,
    },
})

export const loadPlayers = () => {
    return async (dispatch) => {
        let response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli("player list"));
        if (response.status === "error") {
            throw(response.message);
        }
        let players = response.data;
        console.log("players:", players);
        response = JSON.parse(await PlayerWalletModule.call_cli("player mine"));
        if (response.status === "error") {
            throw(response.message);
        }
        const my_players = response.data;
        console.log("my_players:", my_players);
        players.forEach(function (p) { 
            p.pictureUrl = "https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0";
            p.mine = my_players.some(mp => mp === p.name);
        });
        console.debug("loaded players:", players);
        return dispatch(playerSlice.actions.playerAddedMany(players));
    }
}

// might want to actually use createAsyncThunk for this one
export const newPlayer = (name: string, password: Secret<string>) => {
    return async (dispatch, getState) => {
// confusing because of native module. what should its signature in typescript be?
        let response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli_with_password(`player register "${name}"`, password.expose_secret())); 
        if (response.status === "error") {
            throw(response.message);
        }
        if (playerSelectors.selectById(getState(), name)) {
            return dispatch(playerSlice.actions.playerUpdated({
                id: name,
                changes: { mine: true },
            }));
        } else {
            return dispatch(playerSlice.actions.playerAdded({
                name: name,
                mine: true,
// TODO: set portrait based on player name hash
                pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0",
            }));
        }
    }
} 

export const addPlayer = (name: string) => {
    return async (dispatch) => {
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
    }
} 

export const removePlayer = (player: Player) => {
    return async (dispatch) => {
        let cli_response = await PlayerWalletModule.call_cli(`player remove "${player.name}"`);
        if (cli_response === "remove player") {
            return dispatch(playerSlice.actions.playerRemoved(player.name));
        } else {
            throw(cli_response);
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
        let response = JSON.parse(await PlayerWalletModule.call_cli("contract list"));
        if (response.status === "error") {
            throw(response.message);
        }
        let contracts = response.data;
        console.debug("loaded contracts:", contracts);
        return dispatch(contractSlice.actions.contractAddedMany(contracts));
    }
}

export const newContract = (p1Name: string, p2Name: string, sats: number) => {
    return async (dispatch, getState) => {
        let output = await PlayerWalletModule.call_cli(`contract new "${p1Name}" "${p2Name}" ${sats}`); 
        let response = JSON.parse(output);
        if (response.status === "error") {
            throw(response.message)
        }
        let contract = response.data;
        return dispatch(contractSlice.actions.contractAdded({
            cxid: contract.cxid,
            p1Name: contract.p1Name,
            p2Name: contract.p2Name,
            amount: contract.amount,
            fundingTx: false,
            p1Sig: false,
            p2Sig: false,
            arbiterSig: false,
            desc: "",
        }))
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
        let output = await PlayerWalletModule.call_cli("balance");
        let balance = +output;
        return dispatch(balanceSlice.actions.setBalance(balance));
    }
}

export const postedSlice = createSlice({
  name: 'posted',
  initialState: 0,
  reducers: {
    setPosted:  (state, action) => action.payload
  }
})

export const setSelectedPlayerPosted = () => {
    return async (dispatch, getState) => {
        let selectedPlayerName = getState().selectedPlayerName;
        let posted = await getPosted(selectedPlayerName);
        return dispatch(postedSlice.actions.setPosted(posted))
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
