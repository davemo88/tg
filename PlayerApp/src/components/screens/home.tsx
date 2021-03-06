import React, { useState, useEffect, } from 'react';
import { useDispatch, useSelector, } from 'react-redux';
import { StackActions } from '@react-navigation/native'
import CheckBox from '@react-native-community/checkbox';
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerNameSlice } from '../../redux';
import { Player, Contract, ContractStatus} from '../../datatypes';
import { getContractStatus } from '../../dump';
import { postContractInfo, setSelectedPlayerPosted } from '../../wallet';

import { Currency } from '../currency';
import { PlayerPortrait } from '../player-portrait';
import { ContractListItem } from '../contract-list-item';


export const Home = ({ navigation }) => {
    const dispatch = useDispatch();
    const selectedPlayerName = useSelector((state) => state.selectedPlayerName);
    const selectedPlayer = useSelector((state) => playerSelectors.selectById(state, selectedPlayerName));
    const balance = useSelector((state) => state.balance);
    const [posted, setPosted] = useState(store.getState().posted);
    const [showResolved, setShowResolved] = useState(false);
// should use effect for this
    const contracts = useSelector(contractSelectors.selectAll)
    .filter((contract, i, a) =>{ return (
      (contract.p1Name === selectedPlayer.name || contract.p2Name === selectedPlayer.name) 
      && 
      (showResolved || getContractStatus(contract) != ContractStatus.Resolved)
    )})

    useEffect(() => {
        if (selectedPlayer) {
            dispatch(setSelectedPlayerPosted())
                .then(() => setPosted(store.getState().posted))
                .catch(error => console.error(error));
        }
    }, []);

    return (
      <View style={styles.home}>
        <View style={{ minWidth: 360, flex:1, alignItems: 'stretch' }}>
          <View style={{ flex: 1, justifyContent: 'flex-start', }}>
          <View>
            <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', backgroundColor: 'white', padding: 5, margin: 5, height: 42, }}>
              <View>
                <Text style={{ fontSize: 20, }}>Player</Text>
              </View>
              <Button 
                title="Change Player"
                onPress={() => navigation.reset({ index:0, routes: [{ name: 'Player Select' }] })}
              />
            </View>
            <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', padding: 5, margin: 5, backgroundColor: 'slategrey', }}>
              <PlayerPortrait name={selectedPlayer.name} pictureUrl={selectedPlayer.pictureUrl} />
              <View style={{ alignItems: 'center' }}>
                <Text>Wallet Balance</Text>
                <Currency amount={balance} />
              </View>
              <View style={{ alignItems: 'center' }}>
                <Text>Posted Balance</Text>
                <Currency amount={posted} />
                <Button
                    title="Post"
                    onPress={() => navigation.push('Post Contract Info') }
                />
              </View>
            </View> 
          </View> 
          </View>
          <View style={{ flex: 3, }}>
            <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', backgroundColor: 'white', padding: 5, margin: 5, height: 42, }}>
              <View>
                <Text style={{ fontSize: 20, }}>Contracts</Text>
              </View>
              <View style={{ flexDirection: 'row' }}>
                <Text>Show Resolved</Text>
                <CheckBox value={showResolved} onValueChange={(newValue) => setShowResolved(newValue)} />
                <Button 
                  title="Receive"
                  onPress={() => navigation.push('Receive Contract') }
                />
                <Button 
                  title="New"
                  onPress={() => navigation.push('New Contract') }
                />
              </View>
            </View>
            <View style={{ padding: 5, }}>
              <FlatList
                data={contracts}
                renderItem={({item}) => <ContractListItem navigation={navigation} contract={item} />}
                keyExtractor={(item) => item.cxid}
              />
            </View>
          </View>
        </View>
      </View>
    );
}

