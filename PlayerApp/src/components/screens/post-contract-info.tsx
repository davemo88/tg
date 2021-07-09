import React, { useState, } from 'react';
import { Switch, FlatList, Button, StyleSheet, Text, TextInput, View, } from 'react-native';
import { useDispatch } from 'react-redux';

import PlayerWalletModule from './../../PlayerWallet';

import { styles } from '../../styles';
import { Player, Contract } from '../../datatypes';
import { store, playerSelectors } from '../../redux';
import { postContractInfo, } from '../../wallet';
import { Secret } from '../../secret';
import { PasswordEntry } from '../password-entry';

export const PostContractInfo = ({ navigation }) => {
    const [password, setPassword] = React.useState(new Secret(""));
    const [posting, setPosting] = useState(false);

    return (
      <View style={styles.container}>
        <View style={{ margin: 10, padding: 10, backgroundColor: 'lightslategrey' }}>
            
          <Text style={{ fontSize: 17 }}>Post your utxos publicly so other players can challenge you</Text>
          <PasswordEntry password={password} setPassword={setPassword} />
          <Button 
            title="Ok" 
            disabled={posting}
            onPress={() => {
                setPosting(true);
                postContractInfo(store.getState().selectedPlayerName, store.getState().balance, password)
                    .then(() => navigation.reset({ index:0, routes: [{ name: 'Home' }] }))
                    .catch(error => console.error(error))
                    .finally(() => setPosting(false));
            } }
          />
        </View>
      </View>
    );
}
