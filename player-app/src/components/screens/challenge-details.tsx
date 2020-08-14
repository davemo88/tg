import React from 'react';
import { Text, Button, View, } from 'react-native';

import { styles } from '../../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, selectedLocalPlayerIdSlice, } from '../../redux.ts';
import { Player, LocalPlayer, Challenge, ChallengeStatus, getChallengeStatus } from '../../datatypes.ts'
import { dismissChallenge } from '../../mock.ts';

import { ChallengeSummary } from '../challenge-summary.tsx';
import { ChallengeAction } from '../challenge-action.tsx';
import { PlayerPortrait } from '../player-portrait.tsx';
import { Arbiter } from '../arbiter.tsx';
import { Currency } from '../currency.tsx';

export const ChallengeDetails = ({ route, navigation }) => {
  const { challengeId } = route.params;
  const challenge = challengeSelectors.selectById(store.getState(), challengeId);
  const selectedLocalPlayer: LocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);
  const playerOne = playerSelectors.selectById(store.getState(), challenge.playerOneId);
  const playerTwo = playerSelectors.selectById(store.getState(), challenge.playerTwoId);

  return (
    <View style={styles.container}>
      <View style={{ flex: 1, alignItems: 'center', justifyContent: 'space-around', }}>
        <View style= {{flexDirection: 'row', justifyContent: 'space-between' }}>
          <View style={{ flex: 1 }}>
            <Text style={{ fontSize: 20 }}>Player One</Text>
            <PlayerPortrait name={playerOne.name} pictureUrl={playerOne.pictureUrl} />
          </View>
          <View style={{ flex: 1 }}>
            <Text style={{ fontSize: 20 }}>Player Two</Text>
            <PlayerPortrait name={playerTwo.name} pictureUrl={playerTwo.pictureUrl} />
          </View>
        </View>
        <View style={{ justifyContent: 'center' }}>
          <ChallengeSummary challenge={challenge} />
        </View>
      </View>
      <View style={{ flex: 1 }}>
        <View style={{ flex: 3, }}>
          <ChallengeAction challenge={challenge} navigation={navigation} />
        </View>
        <View style={{ flex: 1, justifyContent: 'center', }}>
          <Button 
            title="Dismiss Challenge" 
            onPress={() => {
              dismissChallenge(challenge.id);
              navigation.reset({ index:0, routes: [{ name: 'Home', },] });
            } }
          />
        </View>
      </View>
    </View>
  );
}

