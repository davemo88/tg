export interface Player {
  id:               string;
  name:             string;
  pictureUrl:       string;
}

export interface LocalPlayer {
  id:               string,
  playerId:         string,
  balance:          number,
}

export interface Challenge {
  id:               string;
  playerOneId:      string;
  PlayerTwoId:      string;
  pot:              number;
  fundingTx:       bool;
  playerOneSig:     bool;
  playerTwoSig:     bool;
  arbiterSig:       bool;
}

export const isSignedBy = (challenge: Challenge, player: Player): bool => {
  return (
    ((playerOneId === player.id) && playerOneSig)
    ||
    ((playerTwoId === player.id) && playerTwoSig)
  )
}

export enum ChallengeStatus {
  Unsigned,
  Issued,
  Received,
  Accepted,
  Certified,
  Live,
  Resolved, 
  Invalid,
}

// TODO: this function checks the blockchain for the expected payout transactions for the challenge
// if they aren't found then it isn't resolved. if they are invalid then the challenge is invalid
const isResolved = (challenge: Challenge) => {
  return false;
}

export const getChallengeStatus = (selectedPlayerId: string, challenge: Challenge): ChallengeStatus => {
  if (challenge.playerOneSig && challenge.playerTwoSig && challenge.arbiterSig && challenge.fundingTx && isResolved(challenge) ) {
    return ChallengeStatus.Resolved;
  }
  if (challenge.playerOneSig && challenge.playerTwoSig && challenge.arbiterSig && challenge.fundingTx) {
    return ChallengeStatus.Live;
  }
  else if (challenge.playerOneSig && challenge.playerTwoSig && challenge.arbiterSig) {
    return ChallengeStatus.Certified;
  }
  else if (challenge.playerOneSig && challenge.playerTwoSig) {
    return ChallengeStatus.Accepted;
  }
  else if (challenge.playerOneSig && !challenge.arbiterSig) {
    if ((selectedPlayerId === challenge.playerOneId)) {
      return ChallengeStatus.Issued;
    }
    if (selectedPlayerId === challenge.playerTwoId) {
      return ChallengeStatus.Received;
    }
  }
  else if (challenge.playerTwoSig || challenge.arbiterSig) {
    return ChallengeStatus.Invalid;
  }
  else {
    return ChallengeStatus.Unsigned;
  }
}
