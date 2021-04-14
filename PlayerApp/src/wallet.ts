import { LogBox } from 'react-native';
import { nanoid } from '@reduxjs/toolkit';
import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, payoutSelectors, payoutSlice, selectedPlayerNameSlice, } from './redux';
import { Secret } from './secret';
import { Player, Contract, Payout, } from './datatypes';

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
        return Promise.reject(error);
    }
}

export const createContract = (contract: Contract) => {
  store.dispatch
}

export const createPayout = (contract: Contract) => {
  store.dispatch
}

export const signContract = (contract: Contract) => {
  const selectedPlayerName = store.getState().selectedPlayerName;
  let action = {id: contract.cxid, changes: {}};
  if (contract.playerOneName === selectedPlayerName) {
    action.changes.playerOneSig = true;
  }
  else if (contract.playerTwoName === selectedPlayerName) {
    action.changes.playerTwoSig = true;
  }
  store.dispatch(contractSlice.actions.contractUpdated(action));
}

export const signPayout = (payout: Payout) => {
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

export const denyPayoutRequest = (cxid: string) => {
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

export const arbiterSignPayoutRequest = (payout: Payout) => {
  if (payout.payoutToken) {
    store.dispatch(payoutSlice.actions.payoutUpdated({
      id: payout.cxid,
      changes: { arbiterSig: true },
    }));
  }
}
