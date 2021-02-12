import React, { useState, } from 'react';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Button, StyleSheet, Text, TextInput, View, } from 'react-native';
import { store, playerSlice, newPlayer } from '../../redux';
import { useDispatch } from 'react-redux';

import PlayerWalletModule from './../../PlayerWallet';

import { styles } from '../../styles';
import { Player, Contract, ContractStatus } from '../../datatypes';
import { Secret } from '../../secret';
import { PassphraseEntry } from '../passphrase-entry';
import { PlayerPortrait } from '../player-portrait';

export const NewPlayer = ({ navigation }) => {
    const dispatch = useDispatch();
    const [passphrase, setPassphrase] = React.useState(new Secret(""));
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
            
          <PassphraseEntry passphrase={passphrase} setPassphrase={setPassphrase} />
          <Button 
            title="Ok" 
            disabled={registeringPlayer}
            onPress={() => {
                setRegisteringPlayer(true);
                dispatch(newPlayer(playerName, passphrase))
                    .then(
                        success => {
                            setRegisteringPlayer(false);
                            navigation.reset({ index:0, routes: [{ name: 'Player Select' }] });
                        },
// show reason   f  or failure, e.g. name already taken
                        failure => console.log(failure),
                    )
                    .finally(() => setRegisteringPlayer(false));
            } }
          />
        </View>
        </View>
      </View>
    );
}
