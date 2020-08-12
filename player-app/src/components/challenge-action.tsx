import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, selectedLocalPlayerIdSlice, } from '../redux.ts';
import { Player, LocalPlayer, Challenge, ChallengeStatus, } from '../datatypes.ts';
import { getChallengeStatus } from '../dump.ts';

import { Currency } from './currency.tsx';
import { PlayerPortrait } from './player-portrait.tsx';
import { SignatureSwitch } from './signature-switch.tsx';

export const ChallengeAction = (props) => {
  const selectedLocalPlayer: LocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);

  const [isSigned, setIsSigned] = React.useState(false);
// TODO: components!
  return(
    <View style={{ margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
    {
      {
        [ChallengeStatus.Unsigned]: <ActionUnsigned navigation={props.navigation} isSigned={isSigned} setIsSigned={setIsSigned} />,
        [ChallengeStatus.Issued]: <ActionIssued />,
        [ChallengeStatus.Received]: <ActionRecieved navigation={props.navigation} isSigned={isSigned} setIsSigned={setIsSigned} />,
        [ChallengeStatus.Accepted]: <ActionAccepted navigation={props.navigation} />,
        [ChallengeStatus.Certified]: <ActionCertified navigation={props.navigation} />,
        [ChallengeStatus.Live]: <ActionLive navigation={props.navigation} challenge={props.challenge} />,
        [ChallengeStatus.Resolved]: <ActionResolved />,
        [ChallengeStatus.Invalid]: <ActionInvalid />,
      }[getChallengeStatus(selectedLocalPlayer.playerId, props.challenge)]
    }
    </View>
  )
}

const ActionUnsigned = (props) => {
  return (
    <View>
      <SignatureSwitch isSigned={props.isSigned} setIsSigned={props.setIsSigned} />
      <Button 
        disabled={!props.isSigned} 
        title="Issue Challenge" 
        onPress={() => {
          props.navigation.push('Challenge Details', {challengeId: props.challenge.id })
        } }
      />
    </View>
  )
}

const ActionIssued = (props) => {
  return (
    <View>
      <Text>Waiting for Opponent</Text>
    </View>
  )
}

const ActionRecieved = (props) => {
  return (
    <View>
      <SignatureSwitch isSigned={props.isSigned} setIsSigned={props.setIsSigned} />
      <Button 
        disabled={!props.isSigned} 
        title="Accept Challenge" 
        onPress={() => {
          store.dispatch(challengeSlice.actions.challengeUpdated({ 
            id: props.challenge.id,
            changes: { playerTwoSig: true }
          }))
          props.navigation.push('Challenge Details', {challengeId: props.challenge.id })
        } }
      />
    </View>
  )
}

const ActionAccepted = (props) => {
  return (
    <View>
      <Button 
        title="Send to Arbiter" 
        onPress={() => {
          store.dispatch(challengeSlice.actions.challengeUpdated({ 
            id: props.challenge.id,
            changes: { arbiterSig: true }
          }))
          props.navigation.push('Challenge Details', {challengeId: props.challenge.id })
        } }
      />
    </View>
  )
}

const ActionCertified = (props) => {
  return (
    <View>
      <Button 
        title="Broadcast Funding Tx" 
        onPress={() => {
          store.dispatch(challengeSlice.actions.challengeUpdated({ 
            id: props.challenge.id,
            changes: { fundingTx: true }
          }))
          props.navigation.push('Challenge Details', {challengeId: props.challenge.id })
        } }
      />
    </View>
  )
}

const ActionLive = (props) => {
  return (
    <View>
      <Button 
        title="Request Payout" 
        onPress={() => props.navigation.push('Request Payout', { challengeId: props.challenge.id }) }
      />
    </View>
  )
}

const ActionPayoutRequest = (props) => {
  return (
    <View>
      <Text>Payout Request</Text>
    </View>
  )
}

const ActionResolved = (props) => {
  return (
    <View>
      <Text>Resolved Challenge</Text>
    </View>
  )
}

const ActionInvalid = (props) => {
  return (
    <View>
      <Text>Invalid Challenge</Text>
    </View>
  )
}
