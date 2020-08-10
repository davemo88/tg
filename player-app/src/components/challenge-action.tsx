import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, selectedLocalPlayerIdSlice, } from '../redux.ts';
import { Player, LocalPlayer, Challenge, ChallengeStatus, getChallengeStatus } from '../datatypes.ts';

import { Currency } from './currency.tsx';
import { PlayerPortrait } from './player-portrait.tsx';
import { SignatureSwitch } from './signature-switch.tsx';

export const ChallengeAction = (props) => {
  const selectedLocalPlayer: LocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);

  const [isSigned, setIsSigned] = React.useState(false);

  console.log("challenge", props.challenge);
// TODO: components!
  return(
    <View style={{ margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
    {
      {
        [ChallengeStatus.Unsigned]:
          <View>
            <SignatureSwitch isSigned={isSigned} setIsSigned={setIsSigned} />
            <Button 
              disabled={!isSigned} 
              title="Sign Challenge" 
              onPress={() => {
                props.navigation.push('Challenge Details', {challengeId: props.challenge.id })
              } }
            />
          </View>,
        [ChallengeStatus.Issued]:
          <View>
          </View>,
        [ChallengeStatus.Received]:
          <View>
            <SignatureSwitch isSigned={isSigned} setIsSigned={setIsSigned} />
            <Button 
              disabled={!isSigned} 
              title="Accept Challenge" 
              onPress={() => {
                store.dispatch(challengeSlice.actions.challengeUpdated({ 
                  id: props.challenge.id,
                  changes: { playerTwoSig: true }
                }))
                props.navigation.push('Challenge Details', {challengeId: props.challenge.id })
              } }
            />
          </View>,
        [ChallengeStatus.Accepted]:
          <Button 
            title="Send to Arbiter" 
            onPress={() => {
              store.dispatch(challengeSlice.actions.challengeUpdated({ 
                id: props.challenge.id,
                changes: { arbiterSig: true }
              }))
              props.navigation.push('Challenge Details', {challengeId: props.challenge.id })
            } }
          />,
        [ChallengeStatus.Certified]:
          <Button 
            title="Broadcast Funding Tx" 
            onPress={() => {
              store.dispatch(challengeSlice.actions.challengeUpdated({ 
                id: props.challenge.id,
                changes: { fundingTx: true }
              }))
              props.navigation.push('Challenge Details', {challengeId: props.challenge.id })
            } }
          />,
        [ChallengeStatus.Live]:
          <Button 
            title="Request Payout" 
            onPress={() => props.navigation.push('Request Payout', { challengeId: props.challenge.id }) }
          />,
        [ChallengeStatus.Resolved]:
          <View>
            <Text>Resolved Challenge</Text>
          </View>,
        [ChallengeStatus.Invalid]:
          <View>
            <Text>Invalid Challenge</Text>
          </View>,
      }[getChallengeStatus(selectedLocalPlayer.playerId, props.challenge)]
    }
    </View>
  )
}

