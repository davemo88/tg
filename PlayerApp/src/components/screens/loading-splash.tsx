import 'react-native-gesture-handler';
import React, { useEffect } from 'react';
import { useDispatch } from 'react-redux';
import { Text, View, } from 'react-native';

import PlayerWalletModule from '../../PlayerWallet';

import { store, loadPlayers, loadAll } from '../../redux';

export const LoadingSplash = ({ navigation }) => {
    const dispatch = useDispatch();

    useEffect(() => {
        dispatch(loadAll())
            .then(() => {
                    console.debug("loading complete");
                    navigation.reset({ index:0, routes: [{ name: 'Player Select' }] });
                }).catch(error => {
                    if (error === "no seed. initialize wallet first") {
                    navigation.reset({ index:0, routes: [{ name: 'Initialize Wallet' }] });
                    } else {
                        console.error("loading failed:", error)
                    }
                }); 
    }, []);

    return (
        <View>
            <Text>
               Loading . . .
            </Text>
        </View>
    );
}
