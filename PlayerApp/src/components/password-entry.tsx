import React from 'react';
import { Text, TextInput, View, } from 'react-native';

import { Secret } from '../secret';
import { styles } from '../styles';

export interface PasswordEntryProps {
  password: Secret<string>;
  setPassword: (newPassword: Secret<string>) => void;
}

export const PasswordEntry: React.FC<PasswordEntryProps> = (props) => {
    return (
        <View>
            <Text style={{ fontSize: 20, }}>Wallet Password</Text>
            <TextInput 
                onChangeText={text => props.setPassword(new Secret(text))}
                secureTextEntry={true}    
                value={props.password.expose_secret()}
                style={{ borderWidth: 1, margin: 10, padding: 4, width: 200 }}
            />
        </View>
    )
}
