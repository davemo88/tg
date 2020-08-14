import { store, localPlayerSelectors, challengeSelectors, payoutRequestSelectors } from './redux.ts';

import { LocalPlayer, Player, Challenge, ChallengeStatus, PayoutRequest, PayoutRequestStatus, } from './datatypes.ts';


// TODO: this function checks the blockchain for the expected payout transactions for the challenge
// if they aren't found then it isn't resolved. if they are invalid then the challenge is invalid
// TODO: can only tell if challenge is resolved by looking for txs that spend from fundingTx appropriately. suppose there are inappapropriate ones - then we refuse all payout requests?
// TODO: S is whatever the store type is, idk how to get it atm
export const getChallengeStatus = (challenge: Challenge ): ChallengeStatus => {
  const selectedPlayerId = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId).playerId;

  const allSigned: bool = (challenge.playerOneSig && challenge.playerTwoSig && challenge.arbiterSig);

  if (allSigned && challenge.fundingTx) {

    const payoutRequest = payoutRequestSelectors.selectAll(store.getState())
      .filter((pr, i, a) => pr.challengeId === challenge.id ).pop();

    if (payoutRequest) {
      const payoutRequestStatus = getPayoutRequestStatus(payoutRequest);
      switch (+payoutRequestStatus) {
        case PayoutRequestStatus.Unsigned:
          break;
        case PayoutRequestStatus.LocalPlayerSigned:
          return ChallengeStatus.PayoutRequestIssued;
        case PayoutRequestStatus.OtherPlayerSigned:
          return ChallengeStatus.PayoutRequestReceived;
        case PayoutRequestStatus.Live:
          return ChallengeStatus.PayoutRequestLive;
        case PayoutRequestStatus.Resolved:
          return ChallengeStatus.Resolved;
        case PayoutRequestStatus.Invalid:
          return ChallengeStatus.Live;
      }
    }
    else {
      return ChallengeStatus.Live;
    }
  }
  else if (allSigned) {
    return ChallengeStatus.Certified;
  }
  else if (challenge.playerOneSig && challenge.playerTwoSig) {
    return ChallengeStatus.Accepted;
  }
  else if ((challenge.playerOneSig || challenge.playerTwoSig) && !challenge.arbiterSig) {
    return isChallengeSignedBy(challenge, selectedPlayerId) ? ChallengeStatus.Issued : ChallengeStatus.Received;
  }
  else {
    return isUnsigned(challenge) ? Challenge.Unsigned : Challenge.Invalid;
  }
}

export const getPayoutRequestStatus = (payoutRequest: PayoutRequest ): PayoutRequestStatus => {
  const selectedPlayerId = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId).playerId;
  const challenge = challengeSelectors.selectById(store.getState(), payoutRequest.challengeId);
  if (payoutRequest.payoutTx) {
    return PayoutRequestStatus.Resolved;
  }
  else if (
    ((payoutRequest.playerOneSig && payoutRequest.playerTwoSig))
    || 
    ((payoutRequest.playerOneSig || payoutRequest.playerTwoSig) && payoutRequest.arbiterSig) 
  ) {
    return PayoutRequestStatus.Live;
  }
  else if (isPayoutRequestSignedBy(payoutRequest, selectedPlayerId)) {
    return PayoutRequestStatus.LocalPlayerSigned; 
  }
  else if (isPayoutRequestSignedBy(payoutRequest, getOtherPlayerId(selectedPlayerId, challenge))) {
    return PayoutRequestStatus.OtherPlayerSigned; 
  }
  else {
    return isUnsigned(payoutRequest) ? PayoutRequestStatus.Unsigned : PayoutRequestStatus.Invalid;
  }
}

export const isChallengeSignedBy = (challenge: Challenge | PayoutRequest, playerId: PlayerId): bool => {
  return (
    ((challenge.playerOneId === playerId) && challenge.playerOneSig)
    ||
    ((challenge.playerTwoId === playerId) && challenge.playerTwoSig)
  )
}

export const isPayoutRequestSignedBy = (payoutRequest:  PayoutRequest, playerId: PlayerId): bool => {
  const challenge = challengeSelectors.selectById(store.getState(), payoutRequest.challengeId);
  return (
    ((challenge.playerOneId === playerId) && payoutRequest.playerOneSig)
    ||
    ((challenge.playerTwoId === playerId) && payoutRequest.playerTwoSig)
  )
}

export const isUnsigned = (signable: Challenge | PayoutRequest): bool => {
  return !(signable.playerOneSig || signable.playerTwoSig || signable.arbiterSig)
}

export const getOtherPlayerId = (playerId: PlayerId, challenge: Challenge): PlayerId | undefined => {
  if (challenge.playerOneId === playerId) {
    return challenge.playerTwoId;
  }
  else if (challenge.playerTwoId === playerId) {
    return challenge.playerOneId;
  }
}
