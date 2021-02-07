import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerIdSlice, } from '../../redux';
import { Player, Contract, ContractStatus } from '../../datatypes';
import { signContract } from '../../mock';

import { Currency } from '../currency';
import { SignatureSwitch } from '../signature-switch';
import { PlayerPortrait } from '../player-portrait';
import { PlayerSelector } from '../player-selector';

// TODO: add referee / game-domain expertise delegate pubkey input
export const NewContract = ({ navigation }) => {
  const selectedPlayer = playerSelectors.selectById(store.getState(), store.getState().selectedPlayerId);
  const playerTwos = playerSelectors
    .selectAll(store.getState())
    .filter((player: Player, i, a) => player.id != selectedPlayer.id);
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
      <PlayerSelector selectedPlayerId={playerTwoId} setSelectedPlayerId={setPlayerTwoId} playerIds={playerTwos.map((p: Player) => p.id)} />
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
      <View style={{ flexDirection: 'row', margin: 60 }}>
        <SignatureSwitch isSigned={isSigned} setIsSigned={setIsSigned} />
        <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
          <Button 
            disabled={!valid()}
            title="Issue" 
            onPress={() => {
              store.dispatch(playerSlice.actions.playerUpdated({ 
                id: selectedPlayer.id,
                changes: { balance: selectedPlayer.balance - Math.ceil(contractAmount/2) }
              }))
// native code here
              store.dispatch(contractSlice.actions.contractAdded({ 
                id: nanoid(),
                playerOneId: selectedPlayer.id,
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
