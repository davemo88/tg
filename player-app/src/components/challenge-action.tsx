import React from 'react';
import { nanoid } from '@reduxjs/toolkit';
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, payoutRequestSelectors, payoutRequestSlice, selectedLocalPlayerIdSlice, } from '../redux.ts';
import { Player, LocalPlayer, Challenge, ChallengeStatus, } from '../datatypes.ts';
import { getChallengeStatus } from '../dump.ts';
import { broadcastFundingTx, broadcastPayoutTx, signPayoutRequest, signChallenge, arbiterSignChallenge, declineChallenge, denyPayoutRequest, } from '../mock.ts';

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
        [ChallengeStatus.Received]: <ActionReceived navigation={props.navigation} isSigned={isSigned} setIsSigned={setIsSigned} challenge={props.challenge}/>,
        [ChallengeStatus.Accepted]: <ActionAccepted navigation={props.navigation} challenge={props.challenge} />,
        [ChallengeStatus.Certified]: <ActionCertified navigation={props.navigation} challenge={props.challenge} />,
        [ChallengeStatus.Live]: <ActionLive navigation={props.navigation} challenge={props.challenge} />,
        [ChallengeStatus.PayoutRequestIssued]: <ActionPayoutRequestIssued navigation={props.navigation} challenge={props.challenge} />,
        [ChallengeStatus.PayoutRequestReceived]: <ActionPayoutRequestReceived navigation={props.navigation} challenge={props.challenge} isSigned={isSigned} setIsSigned={setIsSigned} />,
        [ChallengeStatus.PayoutRequestLive]: <ActionPayoutRequestLive navigation={props.navigation} challenge={props.challenge} />,
        [ChallengeStatus.Resolved]: <ActionResolved navigation={props.navigation} challenge={props.challenge}/>,
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
          resetDetails(props.navigation, props.challenge.id);
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

const ActionReceived = (props) => {
  return (
    <View>
      <SignatureSwitch isSigned={props.isSigned} setIsSigned={props.setIsSigned} />
      <Button 
        disabled={!props.isSigned} 
        title="Accept Challenge" 
        onPress={() => {
          signChallenge(props.challenge);
          resetDetails(props.navigation, props.challenge.id);
        } }
      />
      <Button 
        title="Decline Challenge" 
        onPress={() => {
          declineChallenge(props.challenge.id);
          props.navigation.reset({ index:0, routes: [{ name: 'Home', },] });
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
          arbiterSignChallenge(props.challenge);
          resetDetails(props.navigation, props.challenge.id);
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
          broadcastFundingTx(props.challenge);
          resetDetails(props.navigation, props.challenge.id);
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

// TODO: arbitrated payout request how
// 3bools:
// local or other
// tx broadcast or not
// arbiter or not
// then signature state
// pretty big state
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
      <View>
        <Text>Player One Payout: </Text><Currency amount={payoutRequest.playerOneAmount} />
        <Text>Player Two Payout: </Text><Currency amount={payoutRequest.playerTwoAmount} />
        <SignatureSwitch isSigned={props.isSigned} setIsSigned={props.setIsSigned} />
        <Button 
          disabled={!props.isSigned} 
          title='Accept Payout Request'
          onPress={() => {
            signPayoutRequest(payoutRequest)
            resetDetails(props.navigation, props.challenge.id);
          } }
        />
        <Button
          title='Deny Payout Request'
          onPress={() => {
            denyPayoutRequest(payoutRequest.id)
            resetDetails(props.navigation, props.challenge.id);
          } } 
        />
      </View>
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
          resetDetails(props.navigation, props.challenge.id);
        } }
      />
    </View>
  )
}

const ActionResolved = (props) => {
  const payoutRequest = payoutRequestSelectors.selectAll(store.getState())
    .filter((pr, i, a) => pr.challengeId === props.challenge.id ).pop();
  return (
    <View>
      <Text>Resolved Challenge</Text>
      <View style={{ flexDirection: 'row' }}>
        <View>
          <Text>Player One Payout: </Text><Currency amount={payoutRequest.playerOneAmount} />
        </View>
        <View>
          <Text>Player Two Payout: </Text><Currency amount={payoutRequest.playerTwoAmount} />
        </View>
      </View>
      <View style={{ alignItems: 'center' }}>
        <Button 
          title="Home" 
          onPress={() => props.navigation.reset({ index:0, routes: [{ name: 'Home' }] }) } 
        />
      </View>
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

const resetDetails = (navigation, challengeId: ChallengeId) => {
  navigation.reset({ index:0, routes: [{ name: 'Home', }, { name: 'Challenge Details', params: {challengeId: challengeId } }] });
}
