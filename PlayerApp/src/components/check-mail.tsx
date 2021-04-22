import React from 'react';
import { useStore, useDispatch } from 'react-redux';
import { Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../styles';
import { Secret } from '../secret';
import { receiveContract, receivePayout } from '../wallet';

import { PasswordEntry } from './password-entry';

export const CheckMail = (props) => {
    const [checking, setChecking] = React.useState(false);
    const [password, setPassword] = React.useState(new Secret(""));
    const selectedPlayerName = useStore().getState().selectedPlayerName;
    const dispatch = useDispatch();

    return(
        <View style={{ margin: 10 }}>
          <PasswordEntry password={password} setPassword={setPassword} />
          <Button 
            title="Check Mail" 
            disabled={checking}
            onPress={() => {
              setChecking(true);
              Promise.all([
                dispatch(receiveContract(selectedPlayerName, password)),
                dispatch(receivePayout(selectedPlayerName, password)),
              ])
                .then(props.then)
                .catch(error => console.error(error))
                .finally(() => setChecking(false));
            } }
          />
        </View>
    )
}
