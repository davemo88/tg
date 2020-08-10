import React from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, selectedLocalPlayerIdSlice, } from '../../redux.ts';
import { Player, LocalPlayer, Challenge, ChallengeStatus, getChallengeStatus } from '../../datatypes.ts';

import { Currency } from '../currency.tsx';
import { PlayerPortrait } from '../player-portrait.tsx';
import { ChallengeListItem } from '../challenge-list-item.tsx';


export const Home = ({ navigation }) => {
  const selectedLocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);
  const selectedPlayer = playerSelectors.selectById(store.getState(), selectedLocalPlayer.playerId);

  const challenges = challengeSelectors.selectAll(store.getState())
  .filter((challenge, i, a) =>{ return (
    (challenge.playerOneId === selectedLocalPlayer.playerId || challenge.playerTwoId === selectedLocalPlayer.playerId)
  )})
  .filter((challenge, i, a) =>{ return challenge.status != 'Resolved' });

  return (
    <View style={styles.home}>
      <View style={{ minWidth: 360, flex:1, alignItems: 'stretch' }}>
        <View style={{ flex: 1, justifyContent: 'flex-start', }}>
        <View>
          <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', padding: 5, backgroundColor: 'white', padding: 5, margin: 5, height: 42, }}>
            <View>
              <Text style={{ fontSize: 20, }}>Player</Text>
            </View>
            <Button 
              title="Change Player"
              onPress={() => navigation.navigate("Player Select") }
            />
          </View>
          <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', padding: 5, margin: 5, backgroundColor: 'slategrey', }}>
            <PlayerPortrait name={selectedPlayer.name} pictureUrl={selectedPlayer.pictureUrl} />
            <View style={{ alignItems: 'center' }}>
              <Currency amount={selectedLocalPlayer.balance} />
              <Text style={{ textDecorationLine: 'underline', color: 'lightblue' }}>Address</Text>
            </View>
          </View> 
        </View> 
        </View>
        <View style={{ flex: 3, }}>
          <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', padding: 5, backgroundColor: 'white', padding: 5, margin: 5, height: 42, }}>
            <View>
              <Text style={{ fontSize: 20, }}>Challenges</Text>
            </View>
            <View>
              <Button 
                title="New"
                onPress={() => 
                  navigation.navigate('New Challenge')
                }
              />
            </View>
          </View>
          <View style={{ padding: 5, }}>
            <FlatList
              data={challenges}
              renderItem={({item}) => <ChallengeListItem navigation={navigation} challenge={item} />}
            />
          </View>
        </View>
      </View>
    </View>
  );
}

