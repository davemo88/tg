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
                }).catch(error => console.erorr("loading failed:", error)); 
    }, []);

    return (
        <View>
            <Text>
               Loading . . .
            </Text>
        </View>
    );
}
