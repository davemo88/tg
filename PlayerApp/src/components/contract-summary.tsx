import React from 'react';
import { Text, View, } from 'react-native';

import { styles } from '../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerNameSlice, } from '../redux';
import { Player, Contract, ContractStatus, } from '../datatypes'
import { getContractStatus } from '../dump';

import { Currency } from './currency';
import { SigPortrait } from './sig-portrait';
import { ARBITER_NAME, ARBITER_PICTURE_URL } from './arbiter';

export const ContractSummary = (props) => {
  const selectedPlayer: Player = playerSelectors.selectById(store.getState(), store.getState().selectedPlayerName);
  const playerOne = playerSelectors.selectById(store.getState(), props.contract.playerOneName);
  const playerTwo = playerSelectors.selectById(store.getState(), props.contract.playerTwoName);

  return(
    <View style={{ flex: 1 }}>
      <View style={{ alignItems: 'flex-start', padding: 2, }}>
        <Text>Status</Text>
        <Text style={{ fontSize: 15 }}>{ContractStatus[getContractStatus(props.contract)]}</Text>
      </View>
      <View style={{ flexDirection: 'row' }}>
        <View style={{ alignItems: 'flex-start', padding: 2 }}>
          <Text>Pot</Text>
          <Currency amount={props.contract.pot} />
        </View>
        <View style={{ flex: 1, alignItems: 'flex-start', padding: 2, }}>
          <Text>Signatures</Text>
          <View style={{ flex: 1, flexDirection: 'row', justifyContent: 'space-around' }}>
            <SigPortrait name={playerOne.name} pictureUrl={playerOne.pictureUrl} isSigned={props.contract.playerOneSig} />
            <SigPortrait name={playerTwo.name} pictureUrl={playerTwo.pictureUrl} isSigned={props.contract.playerTwoSig} />
            <SigPortrait name={ARBITER_NAME} pictureUrl={ARBITER_PICTURE_URL} isSigned={props.contract.arbiterSig} />
          </View>
        </View>
      </View>
    </View>
  )
}

