import React, { useState, useEffect } from 'react';
import { Text, View, } from 'react-native';
import { useSelector, } from 'react-redux';

import { styles } from '../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, } from '../redux';
import { Player, Contract, ContractStatus, } from '../datatypes'
import { getContractStatus } from '../dump';

import { Currency } from './currency';
import { SigPortrait } from './sig-portrait';
import { ARBITER_NAME, ARBITER_PICTURE_URL } from './arbiter';

export const ContractSummary = (props) => {
  const playerOne = useSelector((state) => playerSelectors.selectById(state, props.contract.p1Name));
  const playerTwo = useSelector((state) => playerSelectors.selectById(state, props.contract.p2Name));
  const [contractStatus, setContractStatus] = useState<string|null>(null);

  useEffect(() => {
      setContractStatus(ContractStatus[getContractStatus(props.contract)]);
  });

  return(
    <View>
      <View style={{ flex:1, alignItems: 'flex-start' }}>
        <Text>{props.contract.desc}</Text>
        <Text style={{ fontSize: 15 }}>Status: {contractStatus}</Text>
      </View>
      <View style={{ flex:1, flexDirection: 'row', justifyContent: 'space-between' }}>
        <View style={{ flex: 1 }}>
          <Text>Amount</Text>
          <Currency amount={props.contract.amount} />
        </View>
        <View style={{ flex: 1, }}>
          <Text>Signatures</Text>
          <View style={{ flex: 1, flexDirection: 'row', justifyContent: 'space-around' }}>
            <SigPortrait name={playerOne.name} pictureUrl={playerOne.pictureUrl} isSigned={props.contract.p1Sig} />
            <SigPortrait name={playerTwo.name} pictureUrl={playerTwo.pictureUrl} isSigned={props.contract.p2Sig} />
            <SigPortrait name={ARBITER_NAME} pictureUrl={ARBITER_PICTURE_URL} isSigned={props.contract.arbiterSig} />
          </View>
        </View>
      </View>
    </View>
  )
}

