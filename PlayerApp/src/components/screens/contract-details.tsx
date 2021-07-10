import React from 'react';
import { useDispatch } from 'react-redux';
import { Text, Button, View, } from 'react-native';

import { styles } from '../../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, payoutSelectors, selectedPlayerNameSlice, } from '../../redux';
import { Player, Contract } from '../../datatypes'
import { dismissContract } from '../../mock';
import { sendContract, deletePayout } from '../../wallet';

import { ContractSummary } from '../contract-summary';
import { ContractAction } from '../contract-action';
import { PlayerPortrait } from '../player-portrait';
import { Arbiter } from '../arbiter';
import { Currency } from '../currency';

export const ContractDetails = ({ route, navigation }) => {
  const dispatch = useDispatch();
  const { cxid } = route.params;
  const contract = contractSelectors.selectById(store.getState(), cxid);
  const payout = payoutSelectors.selectById(store.getState(), cxid);
  const playerOne = playerSelectors.selectById(store.getState(), contract.p1Name);
  const playerTwo = playerSelectors.selectById(store.getState(), contract.p2Name);
  const [sending, setSending] = React.useState(false);

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
        <View style={{ margin: 10 }}>
          <Button 
            title="Send Contract" 
            disabled={sending}
            onPress={() => {
              setSending(true);
              sendContract(contract)
                .catch(error => console.error(error))
                .finally(() => setSending(false));
            } }
          />
          </View>
        <View style={{ flex: 1, justifyContent: 'center', }}>
          { payout &&
            <Button 
              title="Dismiss Payout" 
              onPress={() => {
                dispatch(deletePayout(payout));
                navigation.reset({ index:0, routes: [{ name: 'Home', },] });
              } }
            />
          }
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

