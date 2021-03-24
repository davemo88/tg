import React from 'react';
import { nanoid } from '@reduxjs/toolkit';
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, payoutRequestSelectors, payoutRequestSlice, selectedPlayerIdSlice, } from '../redux';
import { Player, Contract, ContractStatus, } from '../datatypes';
import { getContractStatus } from '../dump';
import { broadcastFundingTx, broadcastPayoutTx, signPayoutRequest, signContract, arbiterSignContract, declineContract, dismissContract, denyPayoutRequest, } from '../mock';

import { Secret } from '../secret';
import { Currency } from './currency';
import { PlayerPortrait } from './player-portrait';
import { PasswordEntry } from './password-entry';

export const ContractAction = (props) => {
  const [isSigned, setIsSigned] = React.useState(false);
  return(
    <View style={{ margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
    {
      {
        [ContractStatus.Unsigned]: <ActionUnsigned navigation={props.navigation} isSigned={isSigned} setIsSigned={setIsSigned} />,
        [ContractStatus.Issued]: <ActionIssued />,
        [ContractStatus.Received]: <ActionReceived navigation={props.navigation} isSigned={isSigned} setIsSigned={setIsSigned} contract={props.contract}/>,
        [ContractStatus.Accepted]: <ActionAccepted navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.Certified]: <ActionCertified navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.Live]: <ActionLive navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.PayoutRequestIssued]: <ActionPayoutRequestIssued navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.PayoutRequestReceived]: <ActionPayoutRequestReceived navigation={props.navigation} contract={props.contract} isSigned={isSigned} setIsSigned={setIsSigned} />,
        [ContractStatus.PayoutRequestLive]: <ActionPayoutRequestLive navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.Resolved]: <ActionResolved navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.Invalid]: <ActionInvalid navigation={props.navigation} contract={props.contract} />,
      }[getContractStatus(props.contract)]
    }
    </View>
  )
}

const ActionUnsigned = (props) => {
  const [password, setPassword] = React.useState(new Secret(""));

  return (
    <View>
      <PasswordEntry password={password} setPassword={setPassword} />
      <Button 
        disabled={!props.isSigned} 
        title="Issue Contract" 
        onPress={() => {
          signContract(props.contract);
          resetDetails(props.navigation, props.contract.id);
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
  const [password, setPassword] = React.useState(new Secret(""));

  return (
    <View>
      <PasswordEntry password={password} setPassword={setPassword} />
      <Button 
        disabled={!props.isSigned} 
        title="Accept Contract" 
        onPress={() => {
          signContract(props.contract);
          resetDetails(props.navigation, props.contract.id);
        } }
      />
      <Button 
        title="Decline Contract" 
        onPress={() => {
          declineContract(props.contract.id);
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
          arbiterSignContract(props.contract);
          resetDetails(props.navigation, props.contract.id);
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
          broadcastFundingTx(props.contract);
          resetDetails(props.navigation, props.contract.id);
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
        onPress={() => props.navigation.push('Request Payout', { contractId: props.contract.id }) }
      />
    </View>
  )
}

// TODO: arbitrated payout request how
// 3bools:
// mine or other
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
  const [password, setPassword] = React.useState(new Secret(""));
  const payoutRequest = payoutRequestSelectors.selectAll(store.getState())
    .filter((pr, i, a) => pr.contractId === props.contract.id ).pop();
  return (
    <View>
      <Text>Payout Request Received</Text>
      <View>
        <Text>Player One Payout: </Text><Currency amount={payoutRequest.playerOneAmount} />
        <Text>Player Two Payout: </Text><Currency amount={payoutRequest.playerTwoAmount} />
        <PasswordEntry password={password} setPassword={setPassword} />
        <Button 
          disabled={!props.isSigned} 
          title='Accept Payout Request'
          onPress={() => {
            signPayoutRequest(payoutRequest)
            resetDetails(props.navigation, props.contract.id);
          } }
        />
        <Button
          title='Deny Payout Request'
          onPress={() => {
            denyPayoutRequest(payoutRequest.id)
            resetDetails(props.navigation, props.contract.id);
          } } 
        />
      </View>
    </View>
  )
}

const ActionPayoutRequestLive = (props) => {
  const payoutRequest = payoutRequestSelectors.selectAll(store.getState())
    .filter((pr, i, a) => pr.contractId === props.contract.id ).pop();
  return (
    <View>
      <Text>Payout Request Live</Text>
      <Button 
        title="Broadcast Payout Tx" 
        onPress={() => {
          broadcastPayoutTx(payoutRequest);
          resetDetails(props.navigation, props.contract.id);
        } }
      />
    </View>
  )
}

const ActionResolved = (props) => {
  const payoutRequest = payoutRequestSelectors.selectAll(store.getState())
    .filter((pr, i, a) => pr.contractId === props.contract.id ).pop();
  return (
    <View>
      <Text>Resolved Contract</Text>
      <View style={{ flexDirection: 'row' }}>
        <View>
          <Text>Player One Payout: </Text><Currency amount={payoutRequest.playerOneAmount} />
        </View>
        <View>
          <Text>Player Two Payout: </Text><Currency amount={payoutRequest.playerTwoAmount} />
        </View>
      </View>
    </View>
  )
}

const ActionInvalid = (props) => {
  return (
    <View style={{ alignItems: 'center' }}>
      <Text>Invalid Contract</Text>
      <Image style={styles.mediumEmote} source={'https://static-cdn.jtvnw.net/emoticons/v1/22998/2.0'} />
    </View>
  )
}

const resetDetails = (navigation, contractId: ContractId) => {
  navigation.reset({ index:0, routes: [{ name: 'Home', }, { name: 'Contract Details', params: {contractId: contractId } }] });
}
