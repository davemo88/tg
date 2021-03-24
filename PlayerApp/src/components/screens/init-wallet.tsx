import React from 'react';
import { Button, Image, Text, TextInput, View, } from 'react-native';
import { Secret } from '../../secret';
import { NETWORK, TITLE_IMAGE_SOURCE, TEST_IMAGE_SOURCE, LIVE_IMAGE_SOURCE, PASSPHRASE_MIN_LENGTH, } from '../../mock';

import { styles } from '../../styles';
import { initWallet } from '../../wallet';

import { PasswordEntry } from '../password-entry';

export const InitWallet = ({ navigation }) => {
// TODO: is this ok? i have no idea (so probably not)
// use this similar to rust:
// https://www.npmjs.com/package/secret-value
    const [password, setPassword] = React.useState(new Secret(""));
    const [initializing, setInitializing] = React.useState(false);

    return (
        <View style={styles.container}>
            <View style={{ flex: 2, padding: 5, alignItems: 'center' }}>
                <Image style={{ width: 256, height: 126 }} source={{uri: TITLE_IMAGE_SOURCE}} />
                { NETWORK === 'Test' &&
                  <Image style={{ width: 133, height: 45 }} source={{uri: TEST_IMAGE_SOURCE}} />
                }
                { NETWORK === 'Live' &&
                  <Image style={{ width: 133, height: 45 }} source={{uri: LIVE_IMAGE_SOURCE}} />
                }
            </View>
            <View style={{ flex:2, alignItems: 'center' }}>
                <PasswordEntry password={password} setPassword={setPassword} />
                <Button
                    title="Ok"
                    disabled={(password.expose_secret().length < PASSPHRASE_MIN_LENGTH)
                              || initializing }
                    onPress={() => {
                        setInitializing(true); 
                        initWallet(password)
                            .then(
                                success => navigation.reset({ index:0, routes: [{ name: 'Loading Splash' }] }),
                                failure => console.log("failure:", failure),
                            ) 
                            .finally(() => setInitializing(false))
                    } }
                />
            </View>
        </View>
    )
}
