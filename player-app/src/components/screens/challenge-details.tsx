import React from 'react';
import { Text, View, } from 'react-native';

import { styles } from '../../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, selectedLocalPlayerIdSlice, } from '../../redux.ts';
import { Player, LocalPlayer, Challenge, ChallengeStatus, getChallengeStatus } from '../../datatypes.ts'

import { ChallengeSummary } from '../challenge-summary.tsx';
import { ChallengeAction } from '../challenge-action.tsx';
import { PlayerPortrait } from '../player-portrait.tsx';
import { Arbiter } from '../arbiter.tsx';
import { Currency } from '../currency.tsx';

export const ChallengeDetails = ({ route, navigation }) => {
  const { challengeId } = route.params;
  const challenge = challengeSelectors.selectById(store.getState(), challengeId);
  const selectedLocalPlayer: LocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);
  let otherPlayer: Player;
  if (challenge.playerOneId === selectedLocalPlayer.playerId) {
    otherPlayer = playerSelectors.selectById(store.getState(), challenge.playerTwoId);
  }
  else {
    otherPlayer = playerSelectors.selectById(store.getState(), challenge.playerOneId);
  }

  return (
    <View style={styles.container}>
      <View style={{ flex: 2, alignItems: 'center', justifyContent: 'space-around', }}>
        <View style= {{flexDirection: 'row', justifyContent: 'space-between' }}>
          <View style={{ flex: 1 }}>
            <Text style={{ fontSize: 20 }}>Opponent</Text>
            <PlayerPortrait name={otherPlayer.name} pictureUrl={otherPlayer.pictureUrl} />
          </View>
        </View>
        <ChallengeSummary challenge={challenge} />
        <View>
          <Text style={{ fontSize: 20 }}>Arbiter</Text>
          <Arbiter />
        </View>
      </View>
      <View style={{ flex: 1 }}>
        <ChallengeAction challenge={challenge} navigation={navigation} />
      </View>
    </View>
  );
}

