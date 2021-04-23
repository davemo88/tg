import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, payoutSelectors, payoutSlice, selectedPlayerNameSlice, } from './redux';
import { Player, Url, Contract, Payout, } from './datatypes';

// probably still s3 somewhere
export const STATIC_CONTENT_HOST: string = 'https://whatchadoinhere.s3.amazonaws.com/';
// yikes
export const TITLE_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'cc.png'; 
export const TEST_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'test.png'; 
export const LIVE_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'live.png'; 

export const PASSPHRASE_MIN_LENGTH = 12;

// this is appdata
export const NETWORK: string = 'Test';

// delete some local data? set flag in db more likely
export const declineContract = (cxid: string) => {
  store.dispatch(contractSlice.actions.contractRemoved(cxid));
}

export const dismissContract = (cxid: string) => {
  store.dispatch(contractSlice.actions.contractRemoved(cxid));
}

export const denyPayout = (payoutId: string) => {
  store.dispatch(payoutSlice.actions.payoutRemoved(payoutId));
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
      id: payout.id,
      changes: { arbiterSig: true },
    }));
  }
}

//
// below functions need crypto wallet functions
//

export const newPlayer = (playerName: string, pictureUrl: Url) => {
  store.dispatch(playerSlice.actions.playerAdded({ name: playerName, pictureUrl: pictureUrl }));
}

export const createContract = (contract: Contract) => {
  store.dispatch
}

export const createPayout = (contract: Contract) => {
  store.dispatch
}

export const signPayout = (payout: Payout) => {
  const selectedPlayerName = store.getState().selectedPlayerName;
  const contract = contractSelectors.selectById(store.getState(), payout.cxid);
  let action = {id: payout.id, changes: {}};
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
    id: payout.id,
    changes: {
      tx: true,
    }
  }));

}
