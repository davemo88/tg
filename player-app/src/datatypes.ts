export type Player = {
  id:               string;
  name:             string;
  pictureUrl:       string;
}

export type LocalPlayer = {
  id:               string,
  playerId:         string,
  balance:          number,
}

export type Challenge = {
  id:               string;
  playerOneId:      string;
  PlayerTwoId:      string;
  pot:              number;
  funding_tx:       bool;
  playerOneSig:     bool;
  playerTwoSig:     bool;
  arbiterSig:       bool;
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

export const getChallengeStatus = (challenge: Challenge) => {
  if (challenge.playerOneSig && challenge.playerTwoSig && challenge.arbiterSig && challenge.funding_tx) {
    return ChallengeStatus.Live;
  }
  else if (challenge.playerOneSig && challenge.playerTwoSig && challenge.arbiterSig) {
    return ChallengeStatus.Certified;
  }
  else if (challenge.playerOneSig && challenge.playerTwoSig) {
    return ChallengeStatus.Accepted;
  }
  else if ((challenge.PlayerOneSig || challenge.playerTwoSig) && !challenge.arbiterSig) {
    return ChallengeStatus.Issued;
  }
  else if (challenge.arbiterSig) {
    return ChallengeStatus.Invalid;
  }
  return ChallengeStatus.Unsigned;
}
