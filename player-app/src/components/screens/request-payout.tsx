import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';
import Slider from '@react-native-community/slider';

import { styles } from '../../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, selectedLocalPlayerIdSlice, } from '../../redux.ts';
import { Player, LocalPlayer, Challenge, ChallengeStatus, getChallengeStatus } from '../../datatypes.ts';

import { Arbiter } from '../arbiter.tsx';
import { Currency } from '../currency.tsx';
import { PlayerPortrait } from '../player-portrait.tsx';
import { SignatureSwitch } from '../signature-switch.tsx';

export const RequestPayout = ({ route, navigation }) => {
  const { challengeId } = route.params;
  const challenge = challengeSelectors.selectById(store.getState(), challengeId);
  const playerOne = playerSelectors.selectById(store.getState(), challenge.playerOneId)
  const playerTwo = playerSelectors.selectById(store.getState(), challenge.playerTwoId)
  const selectedLocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);
  const [playerOnePayout, setPlayerOnePayout] = React.useState(challenge.pot);
  const [playerTwoPayout, setPlayerTwoPayout] = React.useState(0);
  const [isArbitratedPayout, setIsArbitratedPayout] = React.useState(false);
  const toggleArbitration = () => setIsArbitratedPayout(previousState => !previousState);
  const [arbitrationToken, setArbitrationToken] = React.useState('');
  const [isSigned, setIsSigned] = React.useState(false);

  const valid = () => {
    if (isSigned && (!isArbitratedPayout || (arbitrationToken != ''))) {
      return true;
    }
    return false
  }

  return (
    <View style={styles.container}>
      <View>
        <View style={{ alignItems: 'center' }}>
          <View style={{ flexDirection: 'row', alignItems: 'center' }}>
            <Text style={{ fontSize: 16 }}>Total Pot: </Text>
            <Currency amount={challenge.pot} />
          </View>
          <Text style={{ fontSize: 16 }}>Distribute Pot</Text>
          <View style={{ flexDirection: 'row', }}>
          </View>
          <View style={{ flexDirection: 'row', justifyContent: 'flex-start', padding: 10 }}>
            <View style={{ alignItems: 'center', padding: 5 }}>
              <PlayerPortrait name={playerOne.name} pictureUrl={playerOne.pictureUrl} />
              <Currency amount={ playerOnePayout } />
            </View>
            <View style={{ alignItems: 'center', padding: 5 }}>
              <PlayerPortrait name={playerTwo.name} pictureUrl={playerTwo.pictureUrl} />
              <Currency amount={ playerTwoPayout } />
            </View>
          </View>
          <Slider
            style={{ width: 200, height: 40, padding: 5, margin: 5, }}
            value="0"
            onValueChange={ (value) => {
              const newPLayerOnePayout = Math.floor((1-value) * challenge.pot);
              setPlayerOnePayout(newPLayerOnePayout);
              setPlayerTwoPayout(challenge.pot - newPLayerOnePayout);
            }}
          />
        </View>
      </View>
      <View style={{ padding: 5 }}>
        <Text>Arbitrated Payout</Text>
        <Switch 
          onValueChange={toggleArbitration}
          value={isArbitratedPayout}
        />
      </View>
      { isArbitratedPayout && 
        <View style={{ alignItems: 'center' }}>
          <Text style={{ padding: 2 }}>Arbiter</Text>
          <View>
            <Arbiter />
          </View>
          <Text style={{ padding: 2 }}>Token</Text>
          <TextInput
            value={arbitrationToken}
            onChangeText={text => setArbitrationToken(text)}
            style={{ textAlign: 'center', borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
          />
        </View>
      }
      <View style={{ flexDirection: 'row' }}>
        <SignatureSwitch isSigned={isSigned} setIsSigned={setIsSigned} />
        <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
          <Button 
            disabled={!valid()}
            title="Send" 
            onPress={() => {
              if (selectedLocalPlayer.playerId === challenge.playerOneId) {
                store.dispatch(localPlayerSlice.actions.localPlayerUpdated({ 
                  id: selectedLocalPlayer.id,
                  changes: { balance: selectedLocalPlayer.balance + playerOnePayout }
                }))
              } else if (selectedLocalPlayer.playerId === challenge.playerTwoId) {
                store.dispatch(localPlayerSlice.actions.localPlayerUpdated({ 
                  id: selectedLocalPlayer.id,
                  changes: { balance: selectedLocalPlayer.balance + playerTwoPayout }
                }))
              }
// TODO: can only tell if challenge is resolved by looking for txs that spend from fundingTx appropriately. suppose there are inappapropriate ones - then we refuse all payout requests?
              store.dispatch(challengeSlice.actions.challengeUpdated({ 
                id: challenge.id,
                changes: { status: 'Resolved' }
              }))
              //              navigation.push('Home')
              navigation.reset({ index:0, routes: [{ name: 'Home' }] })
            } }
          />
        </View>
      </View>
    </View>
  )
}

