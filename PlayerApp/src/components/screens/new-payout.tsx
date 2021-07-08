import React, { useState } from 'react';
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';
import { useDispatch } from 'react-redux';
import Slider from '@react-native-community/slider';

import { styles } from '../../styles';
import { Secret } from '../../secret';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerNameSlice, payoutSlice, newPayout, } from '../../redux';
import { Player, Contract, Payout, ContractStatus } from '../../datatypes';

import { Arbiter } from '../arbiter';
import { Currency } from '../currency';
import { PlayerPortrait } from '../player-portrait';

export const NewPayout = ({ route, navigation }) => {
  const dispatch = useDispatch();
  const { cxid } = route.params;
  const contract = contractSelectors.selectById(store.getState(), cxid);
  const playerOne = playerSelectors.selectById(store.getState(), contract.p1Name)
  const playerTwo = playerSelectors.selectById(store.getState(), contract.p2Name)
  const selectedPlayer = playerSelectors.selectById(store.getState(), store.getState().selectedPlayerName);
  const selectedPlayerName = store.getState().selectedPlayerName;
  const [p1Amount, setPlayerOnePayout] = React.useState(contract.amount);
  const [p2Amount, setPlayerTwoPayout] = React.useState(0);
  const [isArbitratedPayout, setIsArbitratedPayout] = React.useState(false);
  const toggleArbitration = () => setIsArbitratedPayout(previousState => !previousState);
  const [arbitrationToken, setArbitrationToken] = React.useState('');
  const [creatingPayout, setCreatingPayout] = useState(false);

  const valid = () => {
    return (!creatingPayout && (!isArbitratedPayout || (arbitrationToken != '')))
  }

  return (
    <View style={styles.container}>
      <View>
        <View style={{ alignItems: 'center' }}>
          <View style={{ flexDirection: 'row', alignItems: 'center' }}>
            <Text style={{ fontSize: 16 }}>Total Pot: </Text>
            <Currency amount={contract.amount} />
          </View>
          <Text style={{ fontSize: 16 }}>Distribute Pot</Text>
          <View style={{ flexDirection: 'row', }}>
          </View>
          <View style={{ flexDirection: 'row', justifyContent: 'flex-start', padding: 10 }}>
            <View style={{ alignItems: 'center', padding: 5 }}>
              <PlayerPortrait name={playerOne.name} pictureUrl={playerOne.pictureUrl} />
              <Currency amount={ p1Amount } />
            </View>
            <View style={{ alignItems: 'center', padding: 5 }}>
              <PlayerPortrait name={playerTwo.name} pictureUrl={playerTwo.pictureUrl} />
              <Currency amount={ p2Amount } />
            </View>
          </View>
          <Slider
            style={{ width: 200, height: 40, padding: 5, margin: 5, }}
            onValueChange={ (value) => {
              const newPlayerOnePayout = Math.floor((1-(+value)) * contract.amount);
              const newPlayerTwoPayout = contract.amount - newPlayerOnePayout;
              setPlayerOnePayout(newPlayerOnePayout);
              setPlayerTwoPayout(newPlayerTwoPayout);
            }}
          />
        </View>
      </View>
      <View style={{ padding: 5, alignItems: 'center' }}>
        <Text>Arbitrated Payout</Text>
        <Switch 
          onValueChange={toggleArbitration}
          value={isArbitratedPayout}
        />
      </View>
      { isArbitratedPayout && 
        <View style={{ alignItems: 'center' }}>
          <Text style={{ padding: 2 }}>Arbiter</Text>
          <View>
            <Arbiter />
          </View>
          <Text style={{ padding: 2 }}>Paste Signed Token</Text>
          <TextInput
            value={arbitrationToken}
            onChangeText={text => setArbitrationToken(text)}
            style={{ borderWidth: 1, margin: 10, padding: 4, width: 120 }}
          />
        </View>
      }
      <View style={{ flexDirection: 'row' }}>
        <View style={{ margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
          <Button 
            disabled={!valid()}
            title="Create" 
            onPress={() => {
              setCreatingPayout(true);
              dispatch(newPayout(contract.cxid, p1Amount, p2Amount))
                  .then(() => navigation.reset({ index:0, routes: [{ name: 'Home' }, { name: 'Contract Details', params: {cxid: contract.cxid } }] }))
                  .catch(error => console.error(error))
                  .finally(() => setCreatingPayout(false));
            } }
          />
        </View>
      </View>
    </View>
  )
}

