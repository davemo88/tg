import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';
import Slider from '@react-native-community/slider';

import { styles } from '../../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, contractSelectors, contractSlice, selectedLocalPlayerIdSlice, payoutRequestSlice, } from '../../redux.ts';
import { Player, LocalPlayer, Contract, PayoutRequest, ContractStatus, getContractStatus } from '../../datatypes.ts';

import { Arbiter } from '../arbiter.tsx';
import { Currency } from '../currency.tsx';
import { PlayerPortrait } from '../player-portrait.tsx';
import { SignatureSwitch } from '../signature-switch.tsx';

export const RequestPayout = ({ route, navigation }) => {
  const { contractId } = route.params;
  const contract = contractSelectors.selectById(store.getState(), contractId);
  const playerOne = playerSelectors.selectById(store.getState(), contract.playerOneId)
  const playerTwo = playerSelectors.selectById(store.getState(), contract.playerTwoId)
  const selectedLocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);
  const [playerOnePayout, setPlayerOnePayout] = React.useState(contract.pot);
  const [playerTwoPayout, setPlayerTwoPayout] = React.useState(0);
  const [isArbitratedPayout, setIsArbitratedPayout] = React.useState(false);
  const toggleArbitration = () => setIsArbitratedPayout(previousState => !previousState);
  const [arbitrationToken, setArbitrationToken] = React.useState('');
  const [isSigned, setIsSigned] = React.useState(false);

  const valid = () => {
    if (isSigned && (!isArbitratedPayout || (arbitrationToken != ''))) {
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
            <Currency amount={contract.pot} />
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
              const newPLayerOnePayout = Math.floor((1-value) * contract.pot);
              setPlayerOnePayout(newPLayerOnePayout);
              setPlayerTwoPayout(contract.pot - newPLayerOnePayout);
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
        <SignatureSwitch isSigned={isSigned} setIsSigned={setIsSigned} />
        <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
          <Button 
            disabled={!valid()}
            title="Send" 
            onPress={() => {
              store.dispatch(payoutRequestSlice.actions.payoutRequestAdded({
                id: nanoid(),
                contractId: contract.id,
                payoutTx: false,
                playerOneSig: (contract.playerOneId === selectedLocalPlayer.playerId),
                playerTwoSig: (contract.playerTwoId === selectedLocalPlayer.playerId),
                arbiterSig: isArbitratedPayout ? true : false,
                payoutScriptSig: isArbitratedPayout ? true : false,
                playerOneAmount: playerOnePayout,
                playerTwoAmount: playerTwoPayout,
              }))
              navigation.reset({ index:0, routes: [{ name: 'Home' }, { name: 'Contract Details', params: {contractId: contract.id } }] });
            } }
          />
        </View>
      </View>
    </View>
  )
}

