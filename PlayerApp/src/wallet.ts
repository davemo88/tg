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
    try {
        let cli_response = await PlayerWalletModule.call_cli_with_password("init", password.expose_secret());
        if (cli_response !== "wallet initialized") {
            throw(cli_response);
        }
        cli_response = await PlayerWalletModule.call_cli("fund");
    } catch(error) {
        return Promise.reject(error)
    }
}

export const postContractInfo = (name: string, amount: number, password: Secret<string>) => {
    return async (dispatch) => {
        try {
            const response: JsonResponse = JSON.parse(await PlayerWalletModule.call_cli_with_password(`player post "${name}" ${amount}`, password.expose_secret()));
            if (response.status === "error") {
                throw(response.message)
            }
            return dispatch(postedSlice.actions.setPosted(response.data))
        } catch(error) {
            return Promise.reject(error)
        }
    }
}

export const getPosted = async (name: string) => {
    try {
        const cli_output = await PlayerWalletModule.call_cli(`player posted "${name}"`);
        let response: JsonResponse = JSON.parse(cli_output);
        if (response.status === "error") {
            throw(response.message);
        }
        const posted = +response.data;
        return Promise.resolve(posted)
    } catch (error) {
        return Promise.reject(error)
    }
}

export const signContract = (contract: Contract, password: Secret<string>) => {
    return async (dispatch) => {
        try {
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
        } catch (error) {
            return Promise.reject(error)
        }
    }
}

export const sendContract = async (contract: Contract) => {
    try {
        const cli_output = await PlayerWalletModule.call_cli(`contract send ${contract.cxid}`);
        const response: JsonResponse = JSON.parse(cli_output);
        if (response.status === "error") {
            throw(response.message);
        }
        return Promise.resolve(null)
    } catch (error) {
        return Promise.reject(error)
    }
}

export const receiveContract = (name: string, password: Secret<string>) => {
    return async (dispatch, getState) => {
        try {
            const cli_output = await PlayerWalletModule.call_cli_with_password(`contract receive ${name}`, password.expose_secret());
            const response: JsonResponse = JSON.parse(cli_output);
            if (response.status === "error") {
                throw(response.message);
            }
            if (contractSelectors.selectById(getState(), contract.cxid)) {
                return dispatch(contractSlice.actions.contractUpdated(contract))
            } else {
                return dispatch(contractSlice.actions.contractAdded(contract))
            }
        } catch (error) {
            return Promise.reject(error)
        }
    }
}

export const createPayout = (contract: Contract) => {
  store.dispatch
}

export const signPayout = (payout: Payout, password: Secret<string>) => {
  const selectedPlayerName = store.getState().selectedPlayerName;
  const contract = contractSelectors.selectById(store.getState(), payout.cxid);
  let action = {id: payout.cxid, changes: {}};
  if (contract.playerOneName === selectedPlayerName) {
    action.changes.playerOneSig = true;
  }
  else if (contract.playerTwoName === selectedPlayerName) {
    action.changes.playerTwoSig = true;
  }
  store.dispatch(payoutSlice.actions.payoutUpdated(action));
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
