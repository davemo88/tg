import 'react-native-gesture-handler';
import React, { useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { Text, View, } from 'react-native';

import PlayerWalletModule from '../../PlayerWallet';

import { loadPlayers, loadAll, playerSelectors } from '../../redux';

export const LoadingSplash = ({ navigation }) => {
    const dispatch = useDispatch();
    let players = useSelector(playerSelectors.selectAll);
    console.debug("loading splash players:", players);

// TODO: if redux store is populated (e.g. after the app is refreshed in the emulator during development), this loading won't do much because it use the `add` and `addMany` actions from `createEntityAdapter`
    useEffect(() => {
        dispatch(loadAll())
            .then(() => {
                    console.debug("loading complete");
                    navigation.reset({ index:0, routes: [{ name: 'Player Select' }] });
                }).catch(error => {
                    if (error.message === "no seed. initialize wallet first") {
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
