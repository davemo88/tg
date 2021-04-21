import React, { useEffect, useState, } from 'react';
import { useDispatch } from 'react-redux';
import { Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles';

import { store, playerSelectors, } from '../../redux';
import { receiveContract } from '../../wallet';

import { Secret } from '../../secret';
import { PasswordEntry } from '../password-entry';

// TODO: add referee / game-domain expertise delegate pubkey input
export const ReceiveContract = ({ navigation }) => {
    const dispatch = useDispatch();
    const selectedPlayer = playerSelectors.selectById(store.getState(), store.getState().selectedPlayerName);
    const playerTwos = playerSelectors
        .selectAll(store.getState())
        .filter((player: Player, i, a) => !player.mine);
    const [checking, setChecking] = React.useState(false);
    const [password, setPassword] = React.useState(new Secret(""));

    return (
        <View style={styles.container}>
          <View style={{ margin: 10 }}>
            <PasswordEntry password={password} setPassword={setPassword} />
            <Button 
              title="Check Mail" 
              disabled={checking}
              onPress={() => {
                setChecking(true);
                dispatch(receiveContract(store.getState().selectedPlayerName, password))
                  .catch(error => console.error(error))
                  .finally(() => setChecking(false));
              } }
            />
          </View>
        </View>
    );
}
