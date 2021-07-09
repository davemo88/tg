import React from 'react';
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { getOtherPlayerName, } from '../dump';
import { styles } from '../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerNameSlice, } from '../redux';
import { Player, Contract } from '../datatypes';

import { Currency } from './currency';
import { PlayerPortrait } from './player-portrait';
import { ContractSummary } from './contract-summary';

export const ContractListItem = (props) => {
  const selectedPlayer: Player = playerSelectors.selectById(store.getState(), store.getState().selectedPlayerName);
  const otherPlayer = playerSelectors.selectById(store.getState(), getOtherPlayerName(selectedPlayer.name, props.contract));

  return (
    <View style={{ flexDirection: 'row', backgroundColor: 'slategrey', margin: 2, padding: 2 }}>
      <View>
        <PlayerPortrait name={otherPlayer.name} pictureUrl={otherPlayer.pictureUrl} />
      </View>
      <View style={{ flex: 2 }}>
        <ContractSummary contract={props.contract} />
      </View>
      <View style={{ flex: 1, alignItems: 'center', justifyContent: 'center', }}>
        <Button 
          title="Details" 
          onPress={() => 
            props.navigation.push('Contract Details', { cxid: props.contract.cxid })
          }
        />
      </View>
    </View>
  );
}


