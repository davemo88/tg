import React from 'react';
import { useDispatch, } from 'react-redux';
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, payoutSelectors, payoutSlice, selectedPlayerNameSlice, } from '../redux';
import { Player, Contract, ContractStatus, } from '../datatypes';
import { sendContract, signContract } from '../wallet';
import { getContractStatus } from '../dump';
import { broadcastFundingTx, broadcastPayoutTx, signPayout, arbiterSignContract, declineContract, dismissContract, denyPayout, } from '../mock';

import { Secret } from '../secret';
import { Currency } from './currency';
import { PlayerPortrait } from './player-portrait';
import { PasswordEntry } from './password-entry';

export const ContractAction = (props) => {
  return(
    <View style={{ margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
    {
      {
        [ContractStatus.Unsigned]: <ActionUnsigned navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.Signed]: <ActionSigned navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.Received]: <ActionReceived navigation={props.navigation} contract={props.contract}/>,
        [ContractStatus.PlayersSigned]: <ActionPlayersSigned navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.Certified]: <ActionCertified navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.Live]: <ActionLive navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.PayoutUnsigned]: <ActionPayoutUnsigned navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.PayoutSigned]: <ActionPayoutSigned navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.PayoutReceived]: <ActionPayoutReceived navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.PayoutLive]: <ActionPayoutLive navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.Resolved]: <ActionResolved navigation={props.navigation} contract={props.contract} />,
        [ContractStatus.Invalid]: <ActionInvalid navigation={props.navigation} contract={props.contract} />,
      }[getContractStatus(props.contract)]
    }
    </View>
  )
}

const ActionUnsigned = (props) => {
    let dispatch = useDispatch();
    let [signing, setSigning] = React.useState(false);
    const [password, setPassword] = React.useState(new Secret(""));

    return (
      <View>
        <PasswordEntry password={password} setPassword={setPassword} />
        <Button 
          title="Sign Contract" 
          onPress={() => {
            setSigning(true);
            dispatch(signContract(props.contract, password))
              .then(
                  success => resetDetails(props.navigation, props.contract.cxid),
                  failure => console.error(failure),
              )
              .finally(setSigning(false));
          } }
        />
      </View>
    )
}

const ActionSigned = (props) => {
    let [sending, setSending] = React.useState(false);
    return (
      <View>
          <Text>Waiting for other player's siganture</Text>
          <Button 
            title="Send Contract" 
            onPress={() => {
              setSending(true);
              sendContract(props.contract)
                .then(
                    success => resetDetails(props.navigation, props.contract.cxid),
                    failure => console.error(failure),
                )
                .finally(() => setSending(false));
            } }
          />
      </View>
    )
}

const ActionReceived = (props) => {
  const [password, setPassword] = React.useState(new Secret(""));

  return (
    <View>
      <PasswordEntry password={password} setPassword={setPassword} />
      <Button 
        title="Accept Contract" 
        onPress={() => {
//          signContract(props.contract);
          resetDetails(props.navigation, props.contract.cxid);
        } }
      />
      <Button 
        title="Decline Contract" 
        onPress={() => {
          declineContract(props.contract.cxid);
          props.navigation.reset({ index:0, routes: [{ name: 'Home', },] });
        } }
      />
    </View>
  )
}

const ActionPlayersSigned = (props) => {
  return (
    <View>
      <Button 
        title="Send to Arbiter" 
        onPress={() => {
          arbiterSignContract(props.contract);
          resetDetails(props.navigation, props.contract.cxid);
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
          resetDetails(props.navigation, props.contract.cxid);
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
        onPress={() => props.navigation.push('Request Payout', { cxid: props.contract.cxid }) }
      />
    </View>
  )
}

const ActionPayoutUnsigned = (props) => {
    let dispatch = useDispatch();
    let [signing, setSigning] = React.useState(false);
    const [password, setPassword] = React.useState(new Secret(""));

    return (
      <View>
        <PasswordEntry password={password} setPassword={setPassword} />
        <Button 
          title="Sign Payout" 
          onPress={() => {
            setSigning(true);
            dispatch(signPayout(props.contract, password))
              .then(
                  success => resetDetails(props.navigation, props.contract.cxid),
                  failure => console.error(failure),
              )
              .finally(setSigning(false));
          } }
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
const ActionPayoutSigned = (props) => {
    const [sending, setSending] = React.useState(false);
    return (
      <View>
          <Text>Waiting for other player's siganture</Text>
          <Button 
            title="Send Contract" 
            onPress={() => {
              setSending(true);
              sendContract(props.contract)
                .then(
                    success => resetDetails(props.navigation, props.contract.cxid),
                    failure => console.error(failure),
                )
                .finally(setSending(false));
            } }
          />
      </View>
    )
}

const ActionPayoutReceived = (props) => {
  const [password, setPassword] = React.useState(new Secret(""));
  const payout = payoutSelectors.selectAll(store.getState())
    .filter((pr, i, a) => pr.cxid === props.contract.cxid ).pop();
  return (
    <View>
      <Text>Payout Received</Text>
      <View>
        <Text>Player One Payout: </Text><Currency amount={payout.playerOneAmount} />
        <Text>Player Two Payout: </Text><Currency amount={payout.playerTwoAmount} />
        <PasswordEntry password={password} setPassword={setPassword} />
        <Button 
          title='Accept Payout'
          onPress={() => {
            signPayout(payout)
            resetDetails(props.navigation, props.contract.cxid);
          } }
        />
        <Button
          title='Deny Payout'
          onPress={() => {
            denyPayout(payout.cxid)
            resetDetails(props.navigation, props.contract.cxid);
          } } 
        />
      </View>
    </View>
  )
}

const ActionPayoutLive = (props) => {
  const payout = payoutSelectors.selectAll(store.getState())
    .filter((pr, i, a) => pr.cxid === props.contract.cxid ).pop();
  return (
    <View>
      <Text>Payout Live</Text>
      <Button 
        title="Broadcast Payout Tx" 
        onPress={() => {
          broadcastPayoutTx(payout);
          resetDetails(props.navigation, props.contract.cxid);
        } }
      />
    </View>
  )
}

const ActionResolved = (props) => {
  const payout = payoutSelectors.selectAll(store.getState())
    .filter((pr, i, a) => pr.cxid === props.contract.cxid ).pop();
  return (
    <View>
      <Text>Resolved Contract</Text>
      <View style={{ flexDirection: 'row' }}>
        <View>
          <Text>Player One Payout: </Text><Currency amount={payout.playerOneAmount} />
        </View>
        <View>
          <Text>Player Two Payout: </Text><Currency amount={payout.playerTwoAmount} />
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

const resetDetails = (navigation, cxid: string) => {
  navigation.reset({ index:0, routes: [{ name: 'Home', }, { name: 'Contract Details', params: {cxid: cxid } }] });
}
