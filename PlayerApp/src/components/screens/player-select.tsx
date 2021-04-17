import React from 'react';
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles';

import { useDispatch } from 'react-redux';
import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerNameSlice, } from '../../redux';
import { Player, Contract, ContractStatus, } from '../../datatypes';
import { NETWORK, TITLE_IMAGE_SOURCE, TEST_IMAGE_SOURCE, LIVE_IMAGE_SOURCE, } from '../../mock';

import { PlayerSelector } from '../player-selector';

export const PlayerSelect = ({ navigation }) => {
    const dispatch = useDispatch();
    const players = playerSelectors.selectAll(store.getState()).filter(p => p.mine);
    const [selectedPlayerName, setSelectedPlayerName] = React.useState(players.length > 0 ? players[0].name: null)

//   TODO: move Test / Live images to navigation header
    return (
        <View style={styles.container}>
            <View style={{ flex: 3 }}>
                <View style={{ flex: 2, padding: 5, alignItems: 'center' }}>
                    <Image style={{ width: 256, height: 126 }} source={{uri: TITLE_IMAGE_SOURCE}} />
                    { NETWORK === 'Test' &&
                      <Image style={{ width: 133, height: 45 }} source={{uri: TEST_IMAGE_SOURCE}} />
                    }
                    { NETWORK === 'Live' &&
                      <Image style={{ width: 133, height: 45 }} source={{uri: LIVE_IMAGE_SOURCE}} />
                    }
                </View>
                { selectedPlayerName !== null ? 
                    <View style = {{ flex: 2, alignItems: 'center' }}>
                        <View style = {{ flex: 1, justifyContent: 'flex-end' }}>
                            <PlayerSelector 
                              selectedPlayerName={selectedPlayerName}
                              setSelectedPlayerName={setSelectedPlayerName}
                              playerNames={players.map(p => p.name)}
                            />
                        </View>
                        <View style={{ flex: 1, width: 60 }}>
                            <Button 
                              title="Ok" 
                              onPress={() => {
                                  const selectedPlayer = players.find(p => p.name === selectedPlayerName);
                                  dispatch(selectedPlayerNameSlice.actions.setSelectedPlayerName(selectedPlayer.name));
                                  navigation.reset({ index:0,   routes: [{ name: 'Home' }] })
                              }   }
                            />
                        </View>
                    </View>
                    : <Text>No Players</Text>
                }
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
    );
}
