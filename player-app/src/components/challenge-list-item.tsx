import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { getOtherPlayerId } from '../dump.ts';
import { styles } from '../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, selectedLocalPlayerIdSlice, } from '../redux.ts';
import { Player, LocalPlayer, Challenge, ChallengeStatus, getChallengeStatus } from '../datatypes.ts';

import { Currency } from './currency.tsx';
import { PlayerPortrait } from './player-portrait.tsx';
import { ChallengeSummary } from './challenge-summary.tsx';

export const ChallengeListItem = (props) => {
  const selectedLocalPlayer: LocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);
  const otherPlayer = playerSelectors.selectById(store.getState(), getOtherPlayerId(selectedLocalPlayer.playerId, props.challenge));

  return (
    <View style={{ flexDirection: 'row', backgroundColor: 'slategrey', margin: 2, padding: 2 }}>
      <PlayerPortrait name={otherPlayer.name} pictureUrl={otherPlayer.pictureUrl} />
      <ChallengeSummary challenge={props.challenge} />
      <View style={{ alignItems: 'center', justifyContent: 'center', }}>
        <View>
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


