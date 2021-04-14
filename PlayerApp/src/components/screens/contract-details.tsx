import React from 'react';
import { Text, Button, View, } from 'react-native';

import { styles } from '../../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerNameSlice, } from '../../redux';
import { Player, Contract, ContractStatus } from '../../datatypes'
import { dismissContract } from '../../mock';

import { ContractSummary } from '../contract-summary';
import { ContractAction } from '../contract-action';
import { PlayerPortrait } from '../player-portrait';
import { Arbiter } from '../arbiter';
import { Currency } from '../currency';

export const ContractDetails = ({ route, navigation }) => {
  const { cxid } = route.params;
  const contract = contractSelectors.selectById(store.getState(), cxid);
  const selectedPlayer: Player = playerSelectors.selectById(store.getState(), store.getState().selectedPlayerName);
  const playerOne = playerSelectors.selectById(store.getState(), contract.playerOneName);
  const playerTwo = playerSelectors.selectById(store.getState(), contract.playerTwoName);

  return (
    <View style={styles.container}>
      <View style={{ flex: 1, alignItems: 'center', justifyContent: 'space-around', }}>
        <View style= {{flexDirection: 'row', justifyContent: 'space-between' }}>
          <View style={{ flex: 1 }}>
            <Text style={{ fontSize: 20 }}>Player One</Text>
            <PlayerPortrait name={playerOne.name} pictureUrl={playerOne.pictureUrl} />
          </View>
          <View style={{ flex: 1 }}>
            <Text style={{ fontSize: 20 }}>Player Two</Text>
            <PlayerPortrait name={playerTwo.name} pictureUrl={playerTwo.pictureUrl} />
          </View>
        </View>
        <View style={{ justifyContent: 'center' }}>
          <ContractSummary contract={contract} />
        </View>
      </View>
      <View style={{ flex: 1 }}>
        <View style={{ flex: 3, }}>
          <ContractAction contract={contract} navigation={navigation} />
        </View>
        <View style={{ flex: 1, justifyContent: 'center', }}>
          <Button 
            title="Dismiss Contract" 
            onPress={() => {
              dismissContract(contract.cxid);
              navigation.reset({ index:0, routes: [{ name: 'Home', },] });
            } }
          />
        </View>
      </View>
    </View>
  );
}

