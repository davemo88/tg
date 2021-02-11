import React from 'react';
import { Button, Image, Text, TextInput, View, } from 'react-native';
import { NETWORK, TITLE_IMAGE_SOURCE, TEST_IMAGE_SOURCE, LIVE_IMAGE_SOURCE, PASSPHRASE_MIN_LENGTH, } from '../../mock';

import { styles } from '../../styles';

import { initWallet } from '../../wallet';

export const InitWallet = ({ navigation }) => {
// TODO: is this ok? i have no idea (so probably not)
// use this similar to rust:
// https://www.npmjs.com/package/secret-value
    const [passphrase, setPassphrase] = React.useState("");
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
                <Text style={{ fontSize: 20, }}>Wallet Passphrase</Text>
                <TextInput
                    onChangeText={text => setPassphrase(text)}
                    secureTextEntry={true}    
                    value={passphrase}
                    style={{ borderWidth: 1, margin: 10, padding: 4, width: 200 }}
                />
                <Button
                    title="Ok"
                    disabled={(passphrase.length < PASSPHRASE_MIN_LENGTH)
                              || initializing }
                    onPress={() => {
                        setInitializing(true); 
                        initWallet(passphrase)
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
