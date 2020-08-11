import React from 'react';
import { Button, Text, View, } from 'react-native';

import { styles } from '../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, selectedLocalPlayerIdSlice, } from '../redux.ts';
import { Player, LocalPlayer, Challenge, ChallengeStatus, getChallengeStatus } from '../datatypes.ts';

export const ChangeLocalPlayer = () => {
  return(
    <View>
      <Button
        title='Change Player'
        onPress={() => {
          store.dispatch(selectedLocalPlayerIdSlice.actions.setSelectedLocalPlayerId('')); 
          
        }}
      />
    </View>
  )
}
