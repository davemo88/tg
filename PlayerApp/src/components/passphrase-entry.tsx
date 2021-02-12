import React from 'react';
import { Text, TextInput, View, } from 'react-native';

import { Secret } from '../secret';
import { styles } from '../styles';

export interface PassphraseEntryProps {
  passphrase: Secret<string>;
  setPassphrase: (newPassphrase: Secret<string>) => void;
}

export const PassphraseEntry: React.FC<PassphraseEntryProps> = (props) => {
    return (
        <View>
            <Text style={{ fontSize: 20, }}>Wallet Passphrase</Text>
            <TextInput 
                onChangeText={text => props.setPassphrase(new Secret(text))}
                secureTextEntry={true}    
                value={props.passphrase.expose_secret()}
                style={{ borderWidth: 1, margin: 10, padding: 4, width: 200 }}
            />
        </View>
    )
}
