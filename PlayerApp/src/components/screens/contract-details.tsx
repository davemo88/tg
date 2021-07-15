import React from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { Text, Button, View, } from 'react-native';

import { styles } from '../../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, payoutSelectors, selectedPlayerNameSlice, } from '../../redux';
import { Player, Contract, ContractStatus } from '../../datatypes'
import { getContractStatus } from '../../dump';
import { dismissContract } from '../../mock';
import { sendContract, deletePayout } from '../../wallet';

import { ContractSummary } from '../contract-summary';
import { PayoutSummary } from '../payout-summary';
import { ContractAction } from '../contract-action';
import { PlayerPortrait } from '../player-portrait';
import { Arbiter } from '../arbiter';
import { Currency } from '../currency';

export const ContractDetails = ({ route, navigation }) => {
  const dispatch = useDispatch();
  const { cxid } = route.params;
  const contract = useSelector((state) => contractSelectors.selectById(state, cxid));
  const payout = useSelector((state) => payoutSelectors.selectById(state, cxid));
  const playerOne = useSelector((state) => playerSelectors.selectById(state, contract.p1Name));
  const playerTwo = useSelector((state) => playerSelectors.selectById(state, contract.p2Name));
  const [sending, setSending] = React.useState(false);

  return (
    <View style={styles.container}>
      <View style={{ flex: 1, alignItems: 'center', borderWidth: 1 }}>
        <View style= {{flexDirection: 'row', justifyContent: 'space-between' }}>
          <View style={{ alignItems: 'center', flex: 1 }}>
            <PlayerPortrait name={playerOne.name} pictureUrl={playerOne.pictureUrl} />
            <Text>{contract.p1TokenDesc}</Text>
          </View>
          <View style={{ alignItems: 'center', flex: 1 }}>
            <PlayerPortrait name={playerTwo.name} pictureUrl={playerTwo.pictureUrl} />
            <Text>{contract.p2TokenDesc}</Text>
          </View>
        </View>
      </View>
      <View style={{ flex: 1, borderWidth: 1 }}>
        <ContractSummary contract={contract} />
      </View>
      { payout && 
        <View style={{ flex: 1, borderWidth: 1}}>
          <PayoutSummary contract={contract} payout={payout}/>
        </View>
      }
      <View style={{ borderWidth: 1}}>
        <ContractAction contract={contract} navigation={navigation} />
      </View>
      <View style={{ alignItems: 'center', borderWidth: 1, justifyContent: 'flex-end'}}>
        <View style={{  }}>
          <View style={{ margin: 3}}>
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
          <View style={{ flexDirection: "row", justifyContent: 'space-between' }}>
            { payout && getContractStatus(contract) != ContractStatus.Resolved &&
              <View style={{ margin: 3}}>
                <Button 
                  title="Dismiss Payout" 
                  onPress={() => {
                    dispatch(deletePayout(payout));
                    navigation.reset({ index:0, routes: [{ name: 'Home', }, { name: 'Contract Details', params: {cxid: cxid} }] });
                  } }
                />
              </View>
            }
            <View style={{ margin: 3}}>
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
      </View>
    </View>
  );
}

