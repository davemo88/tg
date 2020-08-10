import React from 'react';
import { Text, View, } from 'react-native';

import { styles } from '../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, selectedLocalPlayerIdSlice, } from '../redux.ts';
import { Player, LocalPlayer, Challenge, ChallengeStatus, getChallengeStatus } from '../datatypes.ts'

export const ChallengeSummary = (props) => {
  const selectedLocalPlayer: LocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);

  return(
    <View>
      <Text style={{ fontSize: 20 }}>Status</Text>
      <Text>{ChallengeStatus[getChallengeStatus(selectedLocalPlayer.playerId, props.challenge)]}</Text>
    </View>
  )
}

