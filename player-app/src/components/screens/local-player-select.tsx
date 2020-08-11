import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, selectedLocalPlayerIdSlice, } from '../../redux.ts';
import { Player, LocalPlayer, Challenge, ChallengeStatus, getChallengeStatus } from '../../datatypes.ts';

import { PlayerSelector } from '../player-selector.tsx';

export const LocalPlayerSelect = ({ navigation }) => {
  const localPlayers = localPlayerSelectors.selectAll(store.getState());
  const [selectedPlayerId, setSelectedPlayerId] = React.useState(localPlayers[0].playerId)

  return (
    <View style={styles.container}>
      <PlayerSelector 
        selectedPlayerId={selectedPlayerId}
        setSelectedPlayerId={setSelectedPlayerId}
        playerIds={localPlayers.map(l => l.playerId)}
      />
      <View style={{ padding: 10 }}>
        <Button 
          title="Ok" 
          onPress={() => {
            const selectedLocalPlayer = localPlayers.find(l => l.playerId === selectedPlayerId);
            store.dispatch(selectedLocalPlayerIdSlice.actions.setSelectedLocalPlayerId(selectedLocalPlayer.id));
            navigation.reset({ index:0,   routes: [{ name: 'Home' }] })
          } }
        />
      </View>
      <View style={{ padding: 40 }}>
        <Button 
          title="New Local Player" 
          onPress={() => {
            navigation.navigate('New Local Player')
          } }
        />
      </View>
    </View>
  );
}


