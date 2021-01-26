import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles.ts';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerIdSlice, } from '../../redux.ts';
import { Player, Contract, ContractStatus, getContractStatus } from '../../datatypes.ts';
import { newPlayer } from '../../mock.ts';

import { PlayerPortrait } from '../player-portrait.tsx';

import PlayerWalletModule from './../../PlayerWallet';

export const NewPlayer = ({ navigation }) => {
  const [playerName, setPlayerName] = React.useState('');
  const [pictureUrl, setPictureUrl] = React.useState('');

  return (
    <View style={styles.container}>
      <PlayerPortrait name={playerName} pictureUrl={pictureUrl} />
      <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', margin: 10, padding: 10 }}>
        <Text>Player Name</Text>
        <TextInput
          onChangeText={text => setPlayerName(text)}
          value={playerName}
          style={{ borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
        />     
      </View>
      <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', margin: 10, padding: 10 }}>
        <Text>Picture Url</Text>
        <TextInput
          onChangeText={text => setPictureUrl(text)}
          value={pictureUrl}
          style={{ borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
        />     
      </View>
      <View style={{flexDirection: 'row' }}>
      <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey' }}>
        <Button 
          title="Ok" 
          onPress={() => {
            PlayerWalletModule.call_cli(`player register ${playerName}`); 
//            newPlayer(playerName, pictureUrl);
            navigation.reset({ index:0, routes: [{ name: 'Player Select' }] })
          } }
        />
      </View>
      </View>
    </View>
  );
}

