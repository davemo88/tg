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
    return async (dispatch, getState) => {
        const selectedPlayerName = getState().selectedPlayerName;
        let response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli_with_password(`contract sign ${contract.cxid} --sign-funding-tx`, password.expose_secret()));
        if (response.status === "error") {
            throw(response.message);
        }
        let action = {id: contract.cxid, changes: {
            p1Sig: selectedPlayerName === contract.p1Name,
            p2Sig: selectedPlayerName === contract.p2Name,
        }};
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
        if (cxid) {
            cli_output = await PlayerWalletModule.call_cli(`contract summary ${cxid}`);
            response = JSON.parse(cli_output);
            if (response.status === "error") {
                throw(response.message);
            }
            let contract = response.data;
            console.info("received contract:", contract);
            if (contractSelectors.selectById(getState(), cxid)) {
                let action = {id: contract.cxid, changes: {
                    p1Sig: contract.p1Sig,
                    p2Sig: contract.p2Sig,
                    arbiterSig: contract.arbiterSig,
                }};
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
    return async (dispatch, getState) => {
        const selectedPlayerName = getState().selectedPlayerName;
        let response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli_with_password(`payout sign ${payout.cxid}`, password.expose_secret()));
        if (response.status === "error") {
            throw(response.message);
        }
        const contract = contractSelectors.selectById(getState(), payout.cxid);
        if (contract) {
            let action = {id: payout.cxid, changes: {
                p1Sig: selectedPlayerName === contract.p1Name,
                p2Sig: selectedPlayerName === contract.p2Name,
            }};
            return dispatch(payoutSlice.actions.payoutUpdated(action))
        }
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
        let cli_output = await PlayerWalletModule.call_cli_with_password(`payout receive ${name}`, password.expose_secret());
        let response: JsonResponse = JSON.parse(cli_output);
        if (response.status === "error") {
            throw(response.message);
        }
        let cxid = response.data;
        if (cxid) {
            cli_output = await PlayerWalletModule.call_cli(`payout summary ${cxid}`);
            response = JSON.parse(cli_output);
            if (response.status === "error") {
                throw(response.message);
            }
            let payout = response.data;
            console.info("received payout:", payout);
            if (payoutSelectors.selectById(getState(), cxid)) {
                let action = {id: cxid, changes: {
                    p1Amount: payout.p2Amount,
                    p2Amount: payout.p1Amount,
                    p1Sig: payout.p1Sig,
                    p2Sig: payout.p2Sig,
                    arbiterSig: payout.arbiterSig,
                    payoutToken: payout.payoutToken,
                }};
                return dispatch(payoutSlice.actions.payoutUpdated(action))
            } else {
                return dispatch(payoutSlice.actions.payoutAdded(payout))
            }
        }
    }
}

export const broadcastFundingTx = async (contract: Contract) => {
    const cli_output = await PlayerWalletModule.call_cli(`contract broadcast ${contract.cxid}`);
    const response: JsonResponse = JSON.parse(cli_output);
    if (response.status === "error") {
        throw(response.message);
    }
    return Promise.resolve(null)
}

export const broadcastPayoutTx = async (payout: Payout) => {
    const cli_output = await PlayerWalletModule.call_cli(`payout broadcast ${payout.cxid}`);
    const response: JsonResponse = JSON.parse(cli_output);
    if (response.status === "error") {
        throw(response.message);
    }
    return Promise.resolve(null)
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
