import React from 'react';
import { Text, Button, View, } from 'react-native';

import { styles } from '../../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, contractSelectors, contractSlice, selectedLocalPlayerIdSlice, } from '../../redux.ts';
import { Player, LocalPlayer, Contract, ContractStatus, getContractStatus } from '../../datatypes.ts'
import { dismissContract } from '../../mock.ts';

import { ContractSummary } from '../contract-summary.tsx';
import { ContractAction } from '../contract-action.tsx';
import { PlayerPortrait } from '../player-portrait.tsx';
import { Arbiter } from '../arbiter.tsx';
import { Currency } from '../currency.tsx';

export const ContractDetails = ({ route, navigation }) => {
  const { contractId } = route.params;
  const contract = contractSelectors.selectById(store.getState(), contractId);
  const selectedLocalPlayer: LocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);
  const playerOne = playerSelectors.selectById(store.getState(), contract.playerOneId);
  const playerTwo = playerSelectors.selectById(store.getState(), contract.playerTwoId);

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
              dismissContract(contract.id);
              navigation.reset({ index:0, routes: [{ name: 'Home', },] });
            } }
          />
        </View>
      </View>
    </View>
  );
}
