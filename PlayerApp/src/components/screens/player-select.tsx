import React from 'react';
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles';

import { useDispatch, useStore, useSelector } from 'react-redux';
import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerNameSlice, } from '../../redux';
import { Player, Contract, ContractStatus, } from '../../datatypes';
import { NETWORK, TITLE_IMAGE_SOURCE, TEST_IMAGE_SOURCE, LIVE_IMAGE_SOURCE, } from '../../mock';

import { PlayerSelector } from '../player-selector';

// TODO: this Player Select screens sometimes gets out of date,
// e.g. when refreshing the app in the emulator. unless the app is explicitly closed, it won't clear the redux store and so an expired name (i.e. one that used to be mine before the refresh) will still be in the store with `mine: true`.
// in fact the loading function doesn't do much when the app is refreshed this way
// since it tries to add a bunch of objects to the store but objects with those ids likely already exist in the store, so nothing happens
// could use upsert (from redux toolkit createEntityAdapter) in the case of players so that an expired player will have `mine` set to the correct value after an update instead of keeping the old one
// could also explicitly check and update `player.mine` on this page since this is when it's most relevant
// this is mostly a problem on regtest since names only live for 30 blocks 
export const PlayerSelect = ({ navigation }) => {
    const dispatch = useDispatch();
    const store = useStore();
    let players = useSelector(playerSelectors.selectAll);
    players = players.filter(p => p.mine);
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
                { selectedPlayerName ? 
                    <View style = {{ flex: 2, alignItems: 'center' }}>
                        <View style = {{ flex: 1, justifyContent: 'flex-end' }}>
                            <PlayerSelector 
                              selectedPlayerName={selectedPlayerName}
                              setSelectedPlayerName={setSelectedPlayerName}
                              playerNames={players.map(p => p.name)}
                              allowRemoval={false}
                            />
                        </View>
                        <View style={{ flex: 1, width: 60 }}>
                            <Button 
                              title="Ok" 
                              onPress={() => {
                                  dispatch(selectedPlayerNameSlice.actions.setSelectedPlayerName(selectedPlayerName));
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
