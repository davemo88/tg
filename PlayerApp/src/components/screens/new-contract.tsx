import React from 'react';
import { nanoid } from '@reduxjs/toolkit';
import { useDispatch } from 'react-redux';
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerIdSlice, balanceSlice, newContract, } from '../../redux';
import { Player, Contract, ContractStatus } from '../../datatypes';
import { signContract } from '../../mock';

import { Secret } from '../../secret';
import { Currency } from '../currency';
import { PlayerPortrait } from '../player-portrait';
import { PlayerSelector } from '../player-selector';

// TODO: add referee / game-domain expertise delegate pubkey input
export const NewContract = ({ navigation }) => {
    const dispatch = useDispatch();
    const selectedPlayer = playerSelectors.selectById(store.getState(), store.getState().selectedPlayerId);
    const playerTwos = playerSelectors
        .selectAll(store.getState())
        .filter((player: Player, i, a) => !player.mine);
    const [contractAmount, onChangeContractAmount] = React.useState('0');
    const [playerTwoId, setPlayerTwoId] = React.useState(playerTwos.length > 0 ? playerTwos[0].id : null);
    const [creatingContract, setCreatingContract] = React.useState(false);

    const valid = () => {
        if (parseInt(contractAmount) > 0) {
            return true
        }
        return false
    }

    return (
        <View style={styles.container}>
            <Text style={{ fontSize: 20 }}>Choose Player</Text>
            { playerTwoId !== null ? 
                <PlayerSelector selectedPlayerId={playerTwoId} setSelectedPlayerId={setPlayerTwoId} playerIds={playerTwos.map((p: Player) => p.id)} />
                : <Text>No Players</Text>
            }
            <View style={{ margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
                <Button 
                    title="Add Player" 
                    onPress={() => navigation.navigate('Add Player') }
                />
            </View>
            <View style={{ backgroundColor: 'lightslategrey', alignItems: 'center', padding: 10 }}>
                <Text style={{ fontSize: 16 }}>Amount</Text>
                <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', }}>
                    <TextInput
                        onChangeText={text => onChangeContractAmount(text)}
                        onBlur={() => {if (Number.isNaN(parseInt(contractAmount))) { onChangeContractAmount('0')}}}
                        value={contractAmount}
                        style={{ borderWidth: 1, width: 100, margin: 10, padding: 4, textAlign: 'right' }}
                    />         
                </View>
            </View>
            <View style={{ flexDirection: 'row', margin: 60 }}>
                <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
                    <Button 
                        disabled={(!valid() || creatingContract)}
                        title="Create" 
                        onPress={() => {
                            setCreatingContract(true);
                            dispatch(newContract(selectedPlayer.id, playerTwoId, parseInt(contractAmount)))
                                .then(
                                    success => navigation.reset({ index:0, routes: [{ name: 'Home' }] }),
                                    failure => console.log(failure),
                                )
                                .finally(setCreatingContract(false));
                          } }
                    />
                </View>
            </View>
        </View>
    );
}
