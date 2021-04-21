import { LogBox } from 'react-native';
import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, payoutSelectors, payoutSlice, selectedPlayerNameSlice, postedSlice, } from './redux';
import { Secret } from './secret';
import { JsonResponse, Player, Contract, Payout, } from './datatypes';

import PlayerWalletModule from './PlayerWallet';

// probably still s3 somewhere
export const STATIC_CONTENT_HOST: string = 'https://whatchadoinhere.s3.amazonaws.com/';
export const TITLE_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'cc.png'; 
export const TEST_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'test.png'; 
export const LIVE_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'live.png'; 

// this is appdata
export const NETWORK: string = 'Test';

export const initWallet = async (password: Secret<string>) => {
    let cli_response = await PlayerWalletModule.call_cli_with_password("init", password.expose_secret());
    if (cli_response !== "wallet initialized") {
        throw(cli_response);
    }
    cli_response = await PlayerWalletModule.call_cli("fund");
}

export const postContractInfo = (name: string, amount: number, password: Secret<string>) => {
    return async (dispatch) => {
        const response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli_with_password(`player post "${name}" ${amount}`, password.expose_secret()));
        if (response.status === "error") {
            throw(response.message)
        }
        return dispatch(postedSlice.actions.setPosted(response.data))
    }
}

export const getPosted = async (name: string) => {
    const cli_output = await PlayerWalletModule.call_cli(`player posted "${name}"`);
    let response: JsonResponse = JSON.parse(cli_output);
    if (response.status === "error") {
        throw(response.message);
    }
    const posted = +response.data;
    return Promise.resolve(posted)
}

export const signContract = (contract: Contract, password: Secret<string>) => {
    return async (dispatch) => {
        const selectedPlayerName = store.getState().selectedPlayerName;
        let response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli_with_password(`contract sign ${contract.cxid}`, password.expose_secret()));
        if (response.status === "error") {
            throw(response.message);
        }
        let action = {id: contract.cxid, changes: {}};
        if (contract.p1Name === selectedPlayerName) {
            action.changes.p1Sig = true;
        }
        else if (contract.p2Name === selectedPlayerName) {
            action.changes.p2Sig = true;
        }
        return dispatch(contractSlice.actions.contractUpdated(action))
    }
}

export const sendContract = async (contract: Contract) => {
    const cli_output = await PlayerWalletModule.call_cli(`contract send ${contract.cxid}`);
    const response: JsonResponse = JSON.parse(cli_output);
    if (response.status === "error") {
        throw(response.message);
    }
    return Promise.resolve(null)
}

export const receiveContract = (name: string, password: Secret<string>) => {
    return async (dispatch, getState) => {
        let cli_output = await PlayerWalletModule.call_cli_with_password(`contract receive ${name}`, password.expose_secret());
        let response: JsonResponse = JSON.parse(cli_output);
        if (response.status === "error") {
            throw(response.message);
        }
        let cxid = response.data;
        console.info("received contract:", cxid);
        if (cxid) {
            cli_output = await PlayerWalletModule.call_cli(`contract summary ${cxid}`);
            response = JSON.parse(cli_output);
            if (response.status === "error") {
                throw(response.message);
            }
            let contract = response.data;
            console.info("received contract:", contract);
            if (contractSelectors.selectById(getState(), cxid)) {
                let action = {id: contract.cxid, changes: {}};
                action.changes.p1Sig = contract.p1Sig;
                action.changes.p2Sig = contract.p2Sig;
                action.changes.arbiterSig = contract.arbiterSig;
                return dispatch(contractSlice.actions.contractUpdated(action))
            } else {
                return dispatch(contractSlice.actions.contractAdded(contract))
            }
        }
    }
}

export const submitContract = (contract: Contract) => {
    return async (dispatch) => {
        let cli_output = await PlayerWalletModule.call_cli(`contract submit ${contract.cxid}`);
        let response: JsonResponse = JSON.parse(cli_output);
        if (response.status === "error") {
            throw(response.message);
        }
        return dispatch(contractSlice.actions.contractUpdated({
            id: contract.cxid, changes: { arbiterSig: true }
        }))
    }
}

export const createPayout = (contract: Contract) => {
  store.dispatch
}

export const signPayout = (payout: Payout, password: Secret<string>) => {
    return async (dispatch) => {
        const selectedPlayerName = store.getState().selectedPlayerName;
        let response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli_with_password(`payout sign ${payout.cxid}`, password.expose_secret()));
        if (response.status === "error") {
            throw(response.message);
        }
        const contract = contractSelectors.selectById(store.getState(), payout.cxid);
        let action = {id: payout.cxid, changes: {}};
        if (contract.p1Name === selectedPlayerName) {
            action.changes.p1Sig = true;
        }
        else if (contract.p2Name === selectedPlayerName) {
            action.changes.p2Sig = true;
        }
        return dispatch(payoutSlice.actions.payoutUpdated(action))
    }
}

export const sendPayout = async (payout: Payout) => {
    const cli_output = await PlayerWalletModule.call_cli(`payout send ${payout.cxid}`);
    const response: JsonResponse = JSON.parse(cli_output);
    if (response.status === "error") {
        throw(response.message);
    }
    return Promise.resolve(null)
}

export const receivePayout = (name: string, password: Secret<string>) => {
    return async (dispatch, getState) => {
        const cli_output = await PlayerWalletModule.call_cli_with_password(`payout receive ${name}`, password.expose_secret());
        const response: JsonResponse = JSON.parse(cli_output);
        if (response.status === "error") {
            throw(response.message);
        }
        if (payoutSelectors.selectById(getState(), payout.cxid)) {
            return dispatch(payoutSlice.actions.payoutUpdated(payout))
        } else {
            return dispatch(payoutSlice.actions.payoutAdded(payout))
        }
    }
}

export const broadcastFundingTx = (contract: Contract) => {
  store.dispatch(contractSlice.actions.contractUpdated({
    id: contract.cxid,
    changes: {
      fundingTx: true,
    }
  }));
}

export const broadcastPayoutTx = (payout: Payout) => {
  store.dispatch(payoutSlice.actions.payoutUpdated({
    id: payout.cxid,
    changes: {
      payoutTx: true,
    }
  }));
}

// delete some local data? set flag in db more likely
export const declineContract = (cxid: string) => {
  store.dispatch(contractSlice.actions.contractRemoved(cxid));
}

export const dismissContract = (cxid: string) => {
  store.dispatch(contractSlice.actions.contractRemoved(cxid));
}

export const denyPayout = (cxid: string) => {
  store.dispatch(payoutSlice.actions.payoutRemoved(cxid));
}

// arbiter prefixed functions require calls to the arbiter service
export const arbiterSignContract = (contract: Contract) => {
// TODO: validation
  store.dispatch(contractSlice.actions.contractUpdated({
    id: contract.cxid,
    changes: { arbiterSig: true },
  }));
}

export const arbiterSignPayout = (payout: Payout) => {
  if (payout.payoutToken) {
    store.dispatch(payoutSlice.actions.payoutUpdated({
      id: payout.cxid,
      changes: { arbiterSig: true },
    }));
  }
}
