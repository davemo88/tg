import React from 'react';
import { nanoid } from '@reduxjs/toolkit';
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, payoutRequestSelectors, payoutRequestSlice, selectedLocalPlayerIdSlice, } from '../redux.ts';
import { Player, LocalPlayer, Challenge, ChallengeStatus, } from '../datatypes.ts';
import { getChallengeStatus } from '../dump.ts';
import { broadcastFundingTx, broadcastPayoutTx, signPayoutRequest, signChallenge, arbiterSignChallenge, } from '../mock.ts';

import { Currency } from './currency.tsx';
import { PlayerPortrait } from './player-portrait.tsx';
import { SignatureSwitch } from './signature-switch.tsx';

export const ChallengeAction = (props) => {
  const selectedLocalPlayer: LocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);

  const [isSigned, setIsSigned] = React.useState(false);
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
        [ChallengeStatus.PayoutRequestIssued]: <ActionPayoutRequestIssued navigation={props.navigation} challenge={props.challenge} />,
        [ChallengeStatus.PayoutRequestReceived]: <ActionPayoutRequestReceived navigation={props.navigation} challenge={props.challenge} isSigned={isSigned} setIsSigned={setIsSigned} />,
        [ChallengeStatus.PayoutRequestLive]: <ActionPayoutRequestLive navigation={props.navigation} challenge={props.challenge} />,
        [ChallengeStatus.Resolved]: <ActionResolved />,
        [ChallengeStatus.Invalid]: <ActionInvalid />,
      }[getChallengeStatus(props.challenge)]
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
          signChallenge(props.challenge);
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
          signChallenge(props.challenge);
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
          arbiterSignChallenge(challenge);
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
          broadcastFundingTx(challenge);
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

const ActionPayoutRequestIssued = (props) => {
  return (
    <View>
      <Text>Payout Request Issued</Text>
    </View>
  )
}

const ActionPayoutRequestReceived = (props) => {
  const payoutRequest = payoutRequestSelectors.selectAll(store.getState())
    .filter((pr, i, a) => pr.challengeId === props.challenge.id ).pop();
  return (
    <View>
      <Text>Payout Request Received</Text>
      <SignatureSwitch isSigned={props.isSigned} setIsSigned={props.setIsSigned} />
      <Button 
        disabled={!props.isSigned} 
        title="Sign Payout Request" 
        onPress={() => {
          signPayoutRequest(payoutRequest)
          props.navigation.reset({ index:0, routes: [{ name: 'Challenge Details', params: {challengeId: props.challenge.id } }] });
        } }
      />
    </View>
  )
}

const ActionPayoutRequestLive = (props) => {
  const payoutRequest = payoutRequestSelectors.selectAll(store.getState())
    .filter((pr, i, a) => pr.challengeId === props.challenge.id ).pop();
  return (
    <View>
      <Text>Payout Request Live</Text>
      <Button 
        title="Broadcast Payout Tx" 
        onPress={() => {
          broadcastPayoutTx(payoutRequest);
          props.navigation.reset({ index:0, routes: [{ name: 'Home' }] });
        } }
      />
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
