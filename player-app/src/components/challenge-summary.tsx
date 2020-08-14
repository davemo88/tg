import React from 'react';
import { Text, View, } from 'react-native';

import { styles } from '../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, selectedLocalPlayerIdSlice, } from '../redux.ts';
import { Player, LocalPlayer, Challenge, ChallengeStatus, } from '../datatypes.ts'
import { getChallengeStatus } from '../dump.ts';

import { Currency } from './currency.tsx';
import { SigPortrait } from './sig-portrait.tsx';
import { ARBITER_NAME, ARBITER_PICTURE_URL } from './arbiter.tsx';

export const ChallengeSummary = (props) => {
  const selectedLocalPlayer: LocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);
  const playerOne = playerSelectors.selectById(store.getState(), props.challenge.playerOneId);
  const playerTwo = playerSelectors.selectById(store.getState(), props.challenge.playerTwoId);

  return(
    <View style={{ flex: 1 }}>
      <View style={{ alignItems: 'flex-start', padding: 2, }}>
        <Text>Status</Text>
        <Text style={{ fontSize: 15 }}>{ChallengeStatus[getChallengeStatus(props.challenge)]}</Text>
      </View>
      <View style={{ flexDirection: 'row' }}>
        <View style={{ alignItems: 'flex-start', padding: 2 }}>
          <Text>Pot</Text>
          <Currency amount={props.challenge.pot} />
        </View>
        <View style={{ flex: 1, alignItems: 'flex-start', padding: 2, }}>
          <Text>Signatures</Text>
          <View style={{ flex: 1, flexDirection: 'row', justifyContent: 'space-around' }}>
            <SigPortrait name={playerOne.name} pictureUrl={playerOne.pictureUrl} isSigned={props.challenge.playerOneSig} />
            <SigPortrait name={playerTwo.name} pictureUrl={playerTwo.pictureUrl} isSigned={props.challenge.playerTwoSig} />
            <SigPortrait name={ARBITER_NAME} pictureUrl={ARBITER_PICTURE_URL} isSigned={props.challenge.arbiterSig} />
          </View>
        </View>
      </View>
    </View>
  )
}

