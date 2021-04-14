import React from 'react';
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';
import Slider from '@react-native-community/slider';

import { styles } from '../../styles';
import { Secret } from '../../secret';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerNameSlice, payoutSlice, } from '../../redux';
import { Player, Contract, Payout, ContractStatus } from '../../datatypes';

import { Arbiter } from '../arbiter';
import { Currency } from '../currency';
import { PlayerPortrait } from '../player-portrait';
import { PasswordEntry } from '../password-entry';

export const RequestPayout = ({ route, navigation }) => {
  const { cxid } = route.params;
  const contract = contractSelectors.selectById(store.getState(), cxid);
  const playerOne = playerSelectors.selectById(store.getState(), contract.playerOneName)
  const playerTwo = playerSelectors.selectById(store.getState(), contract.playerTwoName)
  const selectedPlayer = playerSelectors.selectById(store.getState(), store.getState().selectedPlayerName);
  const [playerOnePayout, setPlayerOnePayout] = React.useState(contract.amount);
  const [playerTwoPayout, setPlayerTwoPayout] = React.useState(0);
  const [isArbitratedPayout, setIsArbitratedPayout] = React.useState(false);
  const toggleArbitration = () => setIsArbitratedPayout(previousState => !previousState);
  const [arbitrationToken, setArbitrationToken] = React.useState('');
  const [password, setPassword] = React.useState(null);

  const valid = () => {
    if (password !== null && (!isArbitratedPayout || (arbitrationToken != ''))) {
      return true;
    }
    return false
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
              <Currency amount={ playerOnePayout } />
            </View>
            <View style={{ alignItems: 'center', padding: 5 }}>
              <PlayerPortrait name={playerTwo.name} pictureUrl={playerTwo.pictureUrl} />
              <Currency amount={ playerTwoPayout } />
            </View>
          </View>
          <Slider
            style={{ width: 200, height: 40, padding: 5, margin: 5, }}
            value="0"
            onValueChange={ (value) => {
              const newPLayerOnePayout = Math.floor((1-value) * contract.amount);
              setPlayerOnePayout(newPLayerOnePayout);
              setPlayerTwoPayout(contract.amount - newPLayerOnePayout);
            }}
          />
        </View>
      </View>
      <View style={{ padding: 5 }}>
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
          <Text style={{ padding: 2 }}>Token</Text>
          <TextInput
            value={arbitrationToken}
            onChangeText={text => setArbitrationToken(text)}
            style={{ textAlign: 'center', borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
          />
        </View>
      }
      <View style={{ flexDirection: 'row' }}>
        <PasswordEntry password={password} setPassword={setPassword} />
        <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
          <Button 
            disabled={!valid()}
            title="Send" 
            onPress={() => {
              store.dispatch(payoutSlice.actions.payoutAdded({
                cxid: contract.cxid,
                payoutTx: false,
                playerOneSig: (contract.playerOneName === selectedPlayer.name),
                playerTwoSig: (contract.playerTwoName === selectedPlayer.name),
                arbiterSig: isArbitratedPayout ? true : false,
                payoutScriptSig: isArbitratedPayout ? true : false,
                playerOneAmount: playerOnePayout,
                playerTwoAmount: playerTwoPayout,
              }))
              navigation.reset({ index:0, routes: [{ name: 'Home' }, { name: 'Contract Details', params: {cxid: contract.cxid } }] });
            } }
          />
        </View>
      </View>
    </View>
  )
}

