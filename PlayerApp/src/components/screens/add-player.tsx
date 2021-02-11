import React, { useState} from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles';

import { useDispatch } from 'react-redux';
import { store, playerSlice, addPlayer} from '../../redux';
import { Player, Contract, ContractStatus } from '../../datatypes';

export const AddPlayer = ({ navigation }) => {
    const dispatch = useDispatch();
    const [playerName, setPlayerName] = React.useState('');
    const [addingPlayer, setAddingPlayer] = useState(false);

    return (
      <View style={styles.container}>
        <View style={{ flex: 1, alignItems: 'center', margin: 100 }}>
            <Image
              style={styles.mediumEmote}
              source={{uri: "https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0"}}
            />
            <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', margin: 10, padding: 10 }}>
              <Text>Name</Text>
              <View style={{ flex: 1 }}>
                <TextInput
                  onChangeText={text => setPlayerName(text)}
                  value={playerName}
                  style={{ borderWidth: 1, margin: 10, padding: 4, fontSize: 17 }}
                />     
              </View>
            </View>
            <View style={{ margin: 10, marginBottom: 100, padding: 10, backgroundColor: 'lightslategrey' }}>
              <Button 
                  title="Ok" 
                  disabled={addingPlayer}
                  onPress={() => {
                      setAddingPlayer(true);
                      dispatch(addPlayer(playerName))
                          .then(
                              success => navigation.reset({ index:0, routes: [{ name: 'Home' }, { name: 'New Contract' }] }),
                              failure => console.log(failure),
                          )
                          .finally(() => setAddingPlayer(false));
                  } }
              />
            </View>
        </View>
     </View>
    );
}
