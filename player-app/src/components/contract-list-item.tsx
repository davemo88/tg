import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { getOtherPlayerId } from '../dump.ts';
import { styles } from '../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, contractSelectors, contractSlice, selectedLocalPlayerIdSlice, } from '../redux.ts';
import { Player, LocalPlayer, Contract, ContractStatus, getContractStatus } from '../datatypes.ts';

import { Currency } from './currency.tsx';
import { PlayerPortrait } from './player-portrait.tsx';
import { ContractSummary } from './contract-summary.tsx';

export const ContractListItem = (props) => {
  const selectedLocalPlayer: LocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);
  const otherPlayer = playerSelectors.selectById(store.getState(), getOtherPlayerId(selectedLocalPlayer.playerId, props.contract));

  return (
    <View style={{ flexDirection: 'row', backgroundColor: 'slategrey', margin: 2, padding: 2 }}>
      <View>
        <PlayerPortrait name={otherPlayer.name} pictureUrl={otherPlayer.pictureUrl} />
      </View>
      <View style={{ flex: 2 }}>
        <ContractSummary contract={props.contract} />
      </View>
      <View style={{ flex: 1, alignItems: 'center', justifyContent: 'center', }}>
        <Button 
          title="Details" 
          onPress={() => 
            props.navigation.push('Contract Details', { contractId: props.contract.id })
          }
        />
      </View>
    </View>
  );
}


