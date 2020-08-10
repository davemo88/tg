import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, selectedLocalPlayerIdSlice, } from '../redux.ts';
import { Player, LocalPlayer, Challenge, ChallengeStatus, getChallengeStatus } from '../datatypes.ts';

import { Currency } from './currency.tsx';
import { PlayerPortrait } from './player-portrait.tsx';
import { ChallengeSummary } from './challenge-summary.tsx';

export const ChallengeListItem = (props) => {
  const selectedLocalPlayer: LocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);
  let otherPlayer: Player;
  if (props.challenge.playerOneId === selectedLocalPlayer.playerId) {
    otherPlayer = playerSelectors.selectById(store.getState(), props.challenge.playerTwoId);
  }
  else {
    otherPlayer = playerSelectors.selectById(store.getState(), props.challenge.playerOneId);
  }

  return (
    <View style={{ flexDirection: 'row', justifyContent: 'space-between', backgroundColor: 'slategrey', margin: 5, padding: 5 }}>
      <PlayerPortrait name={otherPlayer.name} pictureUrl={otherPlayer.pictureUrl} />
      <View style={{ flexDirection: 'row', padding: 5, margin: 5, alignItems: 'center', justifyContent: 'center', }}>
        <ChallengeSummary challenge={props.challenge} />
        <View>
          <View style={{ padding: 20 }}>
            <Currency amount={props.challenge.pot} />
          </View>
          <Button 
            title="Details" 
            onPress={() => 
              props.navigation.push('Challenge Details', { challengeId: props.challenge.id })
            }
          />
        </View>
      </View>
    </View>
  );
}


