import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, contractSelectors, contractSlice, selectedLocalPlayerIdSlice, } from '../../redux.ts';
import { Player, LocalPlayer, Contract, ContractStatus, getContractStatus } from '../../datatypes.ts';
import { NETWORK, TITLE_IMAGE_SOURCE, TEST_IMAGE_SOURCE, LIVE_IMAGE_SOURCE, } from '../../mock.ts';

import { PlayerSelector } from '../player-selector.tsx';

export const LocalPlayerSelect = ({ navigation }) => {
  const localPlayers = localPlayerSelectors.selectAll(store.getState());
  const [selectedPlayerId, setSelectedPlayerId] = React.useState(localPlayers[0].playerId)

// TODO: move Test / Live images to navigation header
  return (
    <View style={styles.container}>
      <View style={{ flex: 1 }}>
        <View style={{ flex: 2, padding: 5, alignItems: 'center' }}>
          <Image style={{ width: 256, height: 126 }} source={{uri: TITLE_IMAGE_SOURCE}} />
          { NETWORK === 'Test' &&
            <Image style={{ width: 133, height: 45 }} source={{uri: TEST_IMAGE_SOURCE}} />
          }
          { NETWORK === 'Live' &&
            <Image style={{ width: 133, height: 45 }} source={{uri: LIVE_IMAGE_SOURCE}} />
          }
        </View>
        <View style = {{ flex: 1, justifyContent: 'flex-end' }}>
          <PlayerSelector 
            selectedPlayerId={selectedPlayerId}
            setSelectedPlayerId={setSelectedPlayerId}
            playerIds={localPlayers.map(l => l.playerId)}
          />
        </View>
      </View>
      <View style={{ flex: 1 }}>
        <View style={{ flex: 1 }}>
          <Button 
            title="Ok" 
            onPress={() => {
              const selectedLocalPlayer = localPlayers.find(l => l.playerId === selectedPlayerId);
              store.dispatch(selectedLocalPlayerIdSlice.actions.setSelectedLocalPlayerId(selectedLocalPlayer.id));
              navigation.reset({ index:0,   routes: [{ name: 'Home' }] })
            } }
          />
        </View>
        <View style={{ flex: 1 }}>
          <Button 
            title="New Local Player" 
            onPress={() => {
              navigation.navigate('New Local Player')
            } }
          />
        </View>
      </View>
    </View>
  );
}


