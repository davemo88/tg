import React, { useState, } from 'react';
import { Switch, FlatList, Button, StyleSheet, Text, TextInput, View, } from 'react-native';
import { useDispatch } from 'react-redux';

import PlayerWalletModule from './../../PlayerWallet';

import { styles } from '../../styles';
import { Player, Contract, ContractStatus } from '../../datatypes';
import { store, playerSlice, newPlayer } from '../../redux';
import { postContractInfo, } from '../../wallet';
import { Secret } from '../../secret';
import { PasswordEntry } from '../password-entry';
import { PlayerPortrait } from '../player-portrait';

export const NewPlayer = ({ navigation }) => {
    const dispatch = useDispatch();
    const [password, setPassword] = React.useState(new Secret(""));
    const [playerName, setPlayerName] = useState('');
    const [pictureUrl, setPictureUrl] = useState('https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0');
    const [registeringPlayer, setRegisteringPlayer] = useState(false);

    return (
      <View style={styles.container}>
        <PlayerPortrait name={playerName} pictureUrl={pictureUrl} />
        <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', margin: 10, padding: 10 }}>
          <Text>Name</Text>
          <TextInput
            onChangeText={text => setPlayerName(text)}
            value={playerName}
            style={{ borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
          />     
        </View>
        <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', margin: 10, padding: 10 }}>
          <Text>Picture Url</Text>
          <TextInput
            onChangeText={text => setPictureUrl(text)}
            value={pictureUrl}
            style={{ borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
          />
        </View>
        <View style={{flexDirection: 'row' }}>
        <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey' }}>
            
          <PasswordEntry password={password} setPassword={setPassword} />
          <Button 
            title="Ok" 
            disabled={registeringPlayer}
            onPress={() => {
                setRegisteringPlayer(true);
                dispatch(newPlayer(playerName, password))
                    .then(() => {
                            dispatch(postContractInfo(playerName, store.getState().balance, password))
                                .finally(() => navigation.reset({ index:0, routes: [{ name: 'Player Select' }] }));
                        })
                    .catch(error => console.error(error))
                    .finally(() => setRegisteringPlayer(false));
            } }
          />
        </View>
        </View>
      </View>
    );
}
