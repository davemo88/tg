import 'react-native-gesture-handler';
import React, { useEffect } from 'react';
import { useDispatch } from 'react-redux';
import { Text, View, } from 'react-native';

import PlayerWalletModule from '../../PlayerWallet';

import { store, loadPlayers, loadAll } from '../../redux';

export const LoadingSplash = ({ navigation }) => {
//    const [playersLoading, setPlayersLoading] = React.useState(false);
    const dispatch = useDispatch();

    console.log("render loading splash");
//    console.log("players loading:", playersLoading);
    useEffect(() => {
        console.log("using effect");

        dispatch(loadAll())
            .then(
                success => {
                    console.log("loading complete");
                    navigation.reset({ index:0, routes: [{ name: 'Player Select' }] });
                }, 
                failure => {
                    console.error("loading failed:", failure);
                    navigation.reset({ index:0, routes: [{ name: 'Initialize Wallet' }] });
                }
            );
    }, []);

    return (
        <View>
            <Text>
               Loading . . .
            </Text>
        </View>
    );
}
