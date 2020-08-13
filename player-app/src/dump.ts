import { LocalPlayer, Player, Challenge, ChallengeStatus, PayoutRequest, PayoutRequestStatus, } from './datatypes.ts';

import { store, payoutRequestSelectors } from './redux.ts';

// TODO: this function checks the blockchain for the expected payout transactions for the challenge
// if they aren't found then it isn't resolved. if they are invalid then the challenge is invalid
// TODO: can only tell if challenge is resolved by looking for txs that spend from fundingTx appropriately. suppose there are inappapropriate ones - then we refuse all payout requests?
const isResolved = (challenge: Challenge) => {
  return false;
}

// TODO: S is whatever the store type is, idk how to get it atm
export const getChallengeStatus = (selectedPlayerId: string, challenge: Challenge ): ChallengeStatus => {
  if (challenge.playerOneSig && challenge.playerTwoSig && challenge.arbiterSig && challenge.fundingTx) {

    const payoutRequest = payoutRequestSelectors.selectAll(store.getState())
      .filter((pr, i, a) => pr.challengeId === challenge.id );

    if (payoutRequest) {
      const payoutRequestStatus = getPayoutRequestStatus(selectedPlayerId, payoutRequest);
      switch (+payoutRequestStatus) {
        case PayoutRequestStatus.Unsigned:
          break;
        case PayoutRequestStatus.Issued:
          return ChallengeStatus.PayoutRequestIssued;
        case PayoutRequestStatus.Received:
          return ChallengeStatus.PayoutRequestReceived;
        case PayoutRequestStatus.Live:
          return ChallengeStatus.PayoutRequestLive;
        case PayoutRequestStatus.Resolved:
          return ChallengeStatus.Resolved;
        case PayoutRequestStatus.Invalid:
          return ChallengeStatus.Invalid;
      }
    }
    else {
      return ChallengeStatus.Live;
    }
  }
  else if (challenge.playerOneSig && challenge.playerTwoSig && challenge.arbiterSig) {
    return ChallengeStatus.Certified;
  }
  else if (challenge.playerOneSig && challenge.playerTwoSig) {
    return ChallengeStatus.Accepted;
  }
  else if ((challenge.playerOneSig || challenge.playerTwoSig) && !challenge.arbiterSig) {
    if ((selectedPlayerId === challenge.playerOneId)) {
      return ChallengeStatus.Issued;
    }
    if (selectedPlayerId === challenge.playerTwoId) {
      return ChallengeStatus.Received;
    }
  }
  else if (challenge.arbiterSig) {
    return ChallengeStatus.Invalid;
  }
  else {
    return ChallengeStatus.Unsigned;
  }
}

export const getPayoutRequestStatus = (selectedPlayerId: string, payoutRequest: PayoutRequest ): PayoutRequestStatus => {
  if ((
      payoutRequest.playerOneSig && payoutRequest.playerTwoSig 
      ||
      ((payoutRequest.playerOneSig || payoutRequest.playerTwoSig) && payoutRequest.arbiterSig)
  )) {
    return payoutRequest.payoutTx ? PayoutRequestStatus.Resolved : PayoutRequestReceived.Live;
  }
  else if (isSignedBy(payoutRequest, selectedPlayerId)) {
    return PayoutRequestStatus.Issued;
  }
  else if (isSignedBy(payoutRequest, getOtherPlayerId(selectedPlayerId, payoutRequest))) {
    return PayoutRequestStatus.Received;
  }
  else if (!(payoutRequest.playerOneSig || payoutRequest.playerTwoSig || payoutRequest.arbiterSig)) {
    return PayoutRequestStatus.Unsigned;
  }
  else {
    return PayoutRequestStatus.Invalid;
  }
}

export const isSignedBy = (signable: Challenge | PayoutRequest, playerId: PlayerId): bool => {
  return (
    ((signable.playerOneId === playerId) && signable.playerOneSig)
    ||
    ((signable.playerTwoId === playerId) && signable.playerTwoSig)
  )
}

export const getOtherPlayerId = (playerId: string, twoPlayers: Challenge | PayoutRequest) => {
  if (twoPlayers.playerOneId === playerId) {
    return twoPlayers.playerTwoId;
  }
  else if (twoPlayers.playerTwoId === playerId) {
    return twoPlayers.playerOneId;
  }
}
