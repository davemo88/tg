import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, payoutSelectors, payoutSlice, selectedPlayerNameSlice, } from './redux';
import { Player, Url, Contract, Payout, } from './datatypes';

// probably still s3 somewhere
export const STATIC_CONTENT_HOST: string = 'https://whatchadoinhere.s3.amazonaws.com/';
// yikes
export const TITLE_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'cc.png'; 
export const TEST_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'test.png'; 
export const LIVE_IMAGE_SOURCE: string  = STATIC_CONTENT_HOST+'live.png'; 

export const PASSWORD_MIN_LENGTH = 3;

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
