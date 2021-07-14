import React, { useState, useEffect } from 'react';
import { Text, View, } from 'react-native';
import { useSelector, } from 'react-redux';

import { styles } from '../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerNameSlice, } from '../redux';
import { Player, Contract, Payout } from '../datatypes'
import { getContractStatus } from '../dump';

import { Currency } from './currency';
import { SigPortrait } from './sig-portrait';
import { ARBITER_NAME, ARBITER_PICTURE_URL } from './arbiter';

type PayoutSummaryProps = {
    contract: Contract,
    payout: Payout,
}

export const PayoutSummary = (props: PayoutSummaryProps) => {
  const playerOne = useSelector((state) => playerSelectors.selectById(state, props.contract.p1Name));
  const playerTwo = useSelector((state) => playerSelectors.selectById(state, props.contract.p2Name));

  return(
    <View style={{ flex: 1 }}>
      <Text>Payout</Text>
      <View style={{flex: 1, flexDirection: 'row', justifyContent: 'space-between' }}>
        <Text>{playerOne.name}: {props.payout.p1Amount}</Text>
        <Text>{playerTwo.name}: {props.payout.p2Amount}</Text>
      </View>
      <View style={{ flex: 1 }}>
        <Text>Signatures</Text>
      </View>
      <View style={{ flex: 1, flexDirection: 'row', justifyContent: 'space-between' }}>
        <View style={{ flex: 1, flexDirection: 'row', justifyContent: 'space-around' }}>
          <SigPortrait name={playerOne.name} pictureUrl={playerOne.pictureUrl} isSigned={props.payout.p1Sig} />
          <SigPortrait name={playerTwo.name} pictureUrl={playerTwo.pictureUrl} isSigned={props.payout.p2Sig} />
          <SigPortrait name={ARBITER_NAME} pictureUrl={ARBITER_PICTURE_URL} isSigned={props.payout.arbiterSig} />
        </View>
      </View>
    </View>
  )
}


