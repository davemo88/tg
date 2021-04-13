import React from 'react';
import { StackActions } from '@react-navigation/native'
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerIdSlice, } from '../../redux';
import { Player, Contract, ContractStatus} from '../../datatypes';
import { getContractStatus } from '../../dump';

import { Currency } from '../currency';
import { PlayerPortrait } from '../player-portrait';
import { ContractListItem } from '../contract-list-item';


export const Home = ({ navigation }) => {
  const selectedPlayer = playerSelectors.selectById(store.getState(), store.getState().selectedPlayerId);

  const contracts = contractSelectors.selectAll(store.getState())
  .filter((contract, i, a) =>{ return (
// TODO: these ID's aren't persisted; they only exist in redux. need to filter on the names instead
    (contract.playerOneId === selectedPlayer.id || contract.playerTwoId === selectedPlayer.id) 
    && 
    (getContractStatus(contract) != ContractStatus.Resolved)
  )})

  return (
    <View style={styles.home}>
      <View style={{ minWidth: 360, flex:1, alignItems: 'stretch' }}>
        <View style={{ flex: 1, justifyContent: 'flex-start', }}>
        <View>
          <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', backgroundColor: 'white', padding: 5, margin: 5, height: 42, }}>
            <View>
              <Text style={{ fontSize: 20, }}>Player</Text>
            </View>
            <Button 
              title="Change Player"
              onPress={() => navigation.reset({ index:0, routes: [{ name: 'Player Select' }] })}
            />
          </View>
          <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', padding: 5, margin: 5, backgroundColor: 'slategrey', }}>
            <PlayerPortrait name={selectedPlayer.name} pictureUrl={selectedPlayer.pictureUrl} />
            <View style={{ alignItems: 'center' }}>
              <Currency amount={store.getState().balance} />
              <Text style={{ textDecorationLine: 'underline', color: 'lightblue' }}>Address</Text>
            </View>
          </View> 
        </View> 
        </View>
        <View style={{ flex: 3, }}>
          <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', backgroundColor: 'white', padding: 5, margin: 5, height: 42, }}>
            <View>
              <Text style={{ fontSize: 20, }}>Contracts</Text>
            </View>
            <View>
              <Button 
                title="New"
                onPress={() => 
                  navigation.push('New Contract')
                }
              />
            </View>
          </View>
          <View style={{ padding: 5, }}>
            <FlatList
              data={contracts}
              renderItem={({item}) => <ContractListItem navigation={navigation} contract={item} />}
            />
          </View>
        </View>
      </View>
    </View>
  );
}

