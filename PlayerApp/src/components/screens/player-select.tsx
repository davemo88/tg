import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles';

import { useDispatch } from 'react-redux';
import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerIdSlice, } from '../../redux';
import { Player, Contract, ContractStatus, } from '../../datatypes';
import { NETWORK, TITLE_IMAGE_SOURCE, TEST_IMAGE_SOURCE, LIVE_IMAGE_SOURCE, } from '../../mock';

import { PlayerSelector } from '../player-selector';

export const PlayerSelect = ({ navigation }) => {
    const dispatch = useDispatch();
    const players = playerSelectors.selectAll(store.getState());
    const [selectedPlayerId, setSelectedPlayerId] = React.useState(players.length > 0 ? players[0].id : null)

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
                { selectedPlayerId !== null && 
                    <View style = {{ flex: 1, justifyContent: 'flex-end' }}>
                        <PlayerSelector 
                          selectedPlayerId={selectedPlayerId}
                          setSelectedPlayerId={setSelectedPlayerId}
                          playerIds={players.map(p => p.id)}
                        />
                    </View>
                }
            </View>
            <View style={{ flex: 1 }}>
                <View style={{ flex: 1 }}>
                    <Button 
                      title="Ok" 
                      onPress={() => {
                        const selectedPlayer = players.find(p => p.id === selectedPlayerId);
                        dispatch(selectedPlayerIdSlice.actions.setSelectedPlayerId(selectedPlayer.id));
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


