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
//        if (!playersLoading) {
//            setPlayersLoading(true);
//            PlayerWalletModule.call_cli_bytes(unescape(encodeURIcomponent("trolol i am the best weeeeee")));
            console.log(unescape(encodeURIComponent("trolol i am the best weeeeee")));
            dispatch(loadAll())
//                .then(() => {
//                    console.log("loading completed");
//                    navigation.reset({ index:0, routes: [{ name: 'Player Select' }] });
                  .then(
                    success => {
                        console.log("loading completed");
                        navigation.reset({ index:0, routes: [{ name: 'Player Select' }] });
                    }, 
                    failure => {
                        console.log("loading failed");
                        console.log("failure:", failure);
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
