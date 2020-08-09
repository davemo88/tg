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
  funding_tx:       bool;
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
  Accepted,
  Certified,
  Live,
  Resolved, 
  Invalid,
}

export const getChallengeStatus = (challenge: Challenge): ChallengeStatus => {
  if (challenge.playerOneSig && challenge.playerTwoSig && challenge.arbiterSig && challenge.funding_tx) {
    return ChallengeStatus.Live;
  }
  else if (challenge.playerOneSig && challenge.playerTwoSig && challenge.arbiterSig) {
    return ChallengeStatus.Certified;
  }
  else if (challenge.playerOneSig && challenge.playerTwoSig) {
    return ChallengeStatus.Accepted;
  }
  else if ((challenge.playerOneSig || challenge.playerTwoSig) && !challenge.arbiterSig) {
    return ChallengeStatus.Issued;
  }
  else if (challenge.arbiterSig) {
    return ChallengeStatus.Invalid;
  }
  else {
    return ChallengeStatus.Unsigned;
  }
}
