import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles.ts';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerIdSlice, } from '../../redux.ts';
import { Player, Contract, ContractStatus, getContractStatus } from '../../datatypes.ts';
import { NETWORK, TITLE_IMAGE_SOURCE, TEST_IMAGE_SOURCE, LIVE_IMAGE_SOURCE, } from '../../mock.ts';

import { PlayerSelector } from '../player-selector.tsx';

export const PlayerSelect = ({ navigation }) => {
    const players = playerSelectors.selectAll(store.getState());
    console.log("players:", players);
    const [selectedPlayerId, setSelectedPlayerId] = React.useState(players[0].id)

//   TODO: move Test / Live images to navigation header
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
              playerIds={players.map(p => p.id)}
            />
          </View>
        </View>
        <View style={{ flex: 1 }}>
          <View style={{ flex: 1 }}>
            <Button 
              title="Ok" 
              onPress={() => {
                const selectedPlayer = players.find(p => p.id === selectedPlayerId);
                store.dispatch(selectedPlayerIdSlice.actions.setSelectedPlayerId(selectedPlayer.id));
                navigation.reset({ index:0,   routes: [{ name: 'Home' }] })
              } }
            />
          </View>
          <View style={{ flex: 1 }}>
            <Button 
              title="New Player" 
              onPress={() => {
                navigation.navigate('New Player')
              } }
            />
          </View>
        </View>
      </View>
    );
}


