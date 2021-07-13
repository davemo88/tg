import { createStore } from 'redux';
import { createEntityAdapter, createSlice, createReducer, createAction, configureStore, createAsyncThunk } from '@reduxjs/toolkit';
import { Secret, } from './secret';
import { JsonResponse, TxStatus, Player, Contract, Payout, } from './datatypes';

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
            throw Error(response.message);
        }
        let players = response.data;
        console.debug("player list:", players);
        response = JSON.parse(await PlayerWalletModule.call_cli("player mine"));
        if (response.status === "error") {
            throw Error(response.message);
        }
        const my_players = response.data;
        console.debug("player mine:", my_players);
        players.forEach(function (p) { 
//            p.pictureUrl = "https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0";
            p.pictureUrl = `https://avatars.dicebear.com/api/micah/${p.name}.svg`;
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
            throw Error(response.message);
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
//                pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0",
                pictureUrl: `https://avatars.dicebear.com/api/micah/${name}.svg`
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
            throw Error(cli_response);
        }
    }
} 

export const removePlayer = (player: Player) => {
    return async (dispatch) => {
        let cli_response = await PlayerWalletModule.call_cli(`player remove "${player.name}"`);
        if (cli_response === "remove player") {
            return dispatch(playerSlice.actions.playerRemoved(player.name));
        } else {
            throw Error(cli_response);
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
        let response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli("contract list"));
        if (response.status === "error") {
            throw Error(response.message);
        }
        let contracts: Contract[] = response.data;
        for (var idx in contracts) {
            response = JSON.parse(await PlayerWalletModule.call_cli(`get-tx ${contracts[idx].txid}`));
            if (response.status === "error") {
                throw Error(response.message);
            } 
            if (response.data) {
                contracts[idx].txStatus = TxStatus.Confirmed;
            } else {
                contracts[idx].txStatus = TxStatus.Unbroadcast;
            }
        }
        console.debug("loaded contracts:", contracts);
        return dispatch(contractSlice.actions.contractAddedMany(contracts));
    }
}

export const newContract = (p1Name: string, p2Name: string, sats: number, event: Event, eventPayouts: string[]) => {
    return async (dispatch) => {
        let command = `contract new "${p1Name}" "${p2Name}" ${sats} '${JSON.stringify(event, null, 1)}' "${eventPayouts[0]}" "${eventPayouts[1]}"`;
        let output = await PlayerWalletModule.call_cli(command); 
        let response = JSON.parse(output);
        if (response.status === "error") {
            throw Error(response.message)
        }
        let contract = response.data;
        return dispatch(contractSlice.actions.contractAdded({
            cxid: contract.cxid,
            p1Name: contract.p1Name,
            p2Name: contract.p2Name,
            amount: contract.amount,
            p1Sig: false,
            p2Sig: false,
            arbiterSig: false,
            desc: contract.desc,
            txid: contract.txid,
            p1TokenDesc: contract.p1TokenDesc,
            p2TokenDesc: contract.p2TokenDesc,
            txStatus: TxStatus.Unbroadcast,
        }))
    }
}

export const updateContractTxStatus = (contract: Contract) => {
    return async (dispatch) => {
        let response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli(`get-tx ${contract.txid}`));
        if (response.status === "error") {
            throw Error(response.message);
        } 
        if (response.data) {
            return dispatch(contractSlice.actions.contractUpdated({
                id: contract.cxid,
                changes: {
                    txStatus: TxStatus.Confirmed,
                }
            }))
        } 
    }
}

export const loadPayouts = () => {
    return async (dispatch) => {
        let response = JSON.parse(await PlayerWalletModule.call_cli("payout list"));
        if (response.status === "error") {
            throw Error(response.message);
        }
        let payouts: Payout[] = response.data;
        for (var idx in payouts) {
            response = JSON.parse(await PlayerWalletModule.call_cli(`get-tx ${payouts[idx].txid}`));
            if (response.status === "error") {
                throw Error(response.message);
            } 
            if (response.data) {
                payouts[idx].txStatus = TxStatus.Confirmed;
            } else {
// TODO: could be in mempool but we don't have a great way to check yet
                payouts[idx].txStatus = TxStatus.Unbroadcast;
            }
        }
        console.debug("loaded payouts:", payouts);
        return dispatch(payoutSlice.actions.payoutAddedMany(payouts));
    }
}

export const newPayout = (cxid: string, p1Amount: number, p2Amount: number) => {
    return async (dispatch) => {
        console.debug("new payout args:", cxid, p1Amount, p2Amount);
        let response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli(`payout new ${cxid} ${p1Amount} ${p2Amount}`)); 
        console.debug("new payout response:", response);
        if (response.status === "error") {
            throw Error(response.message)
        }
        let payout = response.data;
        return dispatch(payoutSlice.actions.payoutAdded({
            cxid: payout.cxid,
            p1Amount: payout.p1Amount,
            p2Amount: payout.p2Amount,
            p1Sig: false,
            p2Sig: false,
            arbiterSig: false,
            scriptSig: null,
            txid: payout.txid,
            txStatus: TxStatus.Unbroadcast,
        }))
    }
}

export const updatePayoutTxStatus = (payout: Payout) => {
    return async (dispatch) => {
        let response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli(`get-tx ${payout.txid}`));
        if (response.status === "error") {
            throw Error(response.message);
        } 
        if (response.data) {
            return dispatch(payoutSlice.actions.payoutUpdated({
                id: payout.cxid,
                changes: {
                    txStatus: TxStatus.Confirmed,
                }
            }))
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
    payoutAddedMany: payoutAdapter.addMany,
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
        let response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli("balance"));
        if (response.status === "error") {
            throw Error(response.message);
        } 
        let balance = +response.data;
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
            dispatch(loadPayouts()),
        ]);
}
