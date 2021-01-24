import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles.ts';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerIdSlice, } from '../../redux.ts';
import { Player, Contract, ContractStatus, getContractStatus } from '../../datatypes.ts';

export const AddPlayer = ({ navigation }) => {
  const [playerName, setPlayerName] = React.useState('');

  return (
    <View style={styles.container}>
      <Image
        style={styles.mediumEmote}
        source=''
      />
      <View style={{alignItems: 'center', backgroundColor: 'lightslategrey', margin: 10, padding: 10 }}>
        <TextInput
          onChangeText={text => setPlayerName(text)}
          value={playerName}
          style={{ borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
        />     
        <Text>Enter Player Name or Address</Text>
      </View>
      <View style={{flexDirection: 'row' }}>
        <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey' }}>
          <Button 
            title="Ok" 
            onPress={() => {
              const newPlayerId = nanoid();
              store.dispatch(playerSlice.actions.playerAdded({ id: newPlayerId, name: playerName, pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0' }));
              setPlayerName('');
              navigation.reset({ index:0, routes: [{ name: 'Home' }, { name: 'New Contract' }] })
            } }
          />
       </View>
     </View>
   </View>
  );
}
