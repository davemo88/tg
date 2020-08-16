import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, contractSelectors, contractSlice, selectedLocalPlayerIdSlice, } from '../../redux.ts';
import { Player, LocalPlayer, Contract, ContractStatus, getContractStatus } from '../../datatypes.ts';
import { SignContract } from '../../mock.ts';

import { Currency } from '../currency.tsx';
import { SignatureSwitch } from '../signature-switch.tsx';
import { PlayerPortrait } from '../player-portrait.tsx';
import { PlayerSelector } from '../player-selector.tsx';

export const NewContract = ({ navigation }) => {
  const selectedLocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);
  const playerTwos = playerSelectors
    .selectAll(store.getState())
    .filter((player, i, a) => player.id != selectedLocalPlayer.playerId);
  const [contractAmount, onChangeContractAmount] = React.useState('0');
  const [playerTwoId, setPlayerTwoId] = React.useState(playerTwos[0].id);
  const [isSigned, setIsSigned] = React.useState(false);

  const valid = () => {
    if ((parseInt(contractAmount) > 0) && isSigned) {
      return true
    }
    return false
  }

  return (
    <View style={styles.container}>
      <Text style={{ fontSize: 20 }}>Choose Player</Text>
      <PlayerSelector selectedPlayerId={playerTwoId} setSelectedPlayerId={setPlayerTwoId} playerIds={playerTwos.map(p => p.id)} />
      <View style={{ margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
        <Button 
          title="Add Player" 
          onPress={() => navigation.navigate('Add Player') }
        />
      </View>
      <View style={{ backgroundColor: 'lightslategrey', alignItems: 'center', padding: 10 }}>
        <Text style={{ fontSize: 16 }}>Amount</Text>
        <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', }}>
          <TextInput
            onChangeText={text => onChangeContractAmount(text)}
            onBlur={() => {if (Number.isNaN(parseInt(contractAmount))) { onChangeContractAmount('0')}}}
            value={contractAmount}
            style={{ borderWidth: 1, width: 100, margin: 10, padding: 4, textAlign: 'right' }}
          />     
        </View>
      </View>
      <View style={{ flexDirection: 'row' }}>
        <SignatureSwitch isSigned={isSigned} setIsSigned={setIsSigned} />
        <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
          <Button 
            disabled={!valid()}
            title="Issue" 
            onPress={() => {
              store.dispatch(localPlayerSlice.actions.localPlayerUpdated({ 
                id: selectedLocalPlayer.id,
                changes: { balance: selectedLocalPlayer.balance - Math.ceil(contractAmount/2) }
              }))
// native code here
              store.dispatch(contractSlice.actions.contractAdded({ 
                id: nanoid(),
                playerOneId: selectedLocalPlayer.playerId,
                playerTwoId: playerTwoId,
                pot: contractAmount,
                fundingTx: false,
                playerOneSig: true,
                playerTwoSig: false,
                arbiterSig: false,
              }));
              navigation.reset({ index:0, routes: [{ name: 'Home' }] })
            } }
          />
        </View>
      </View>
    </View>
  );
}
