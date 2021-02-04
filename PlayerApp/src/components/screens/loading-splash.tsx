import 'react-native-gesture-handler';
import React, { useEffect } from 'react';
import { useDispatch } from 'react-redux';
import { Text, View, } from 'react-native';

import { store, loadPlayers, playerSlice, } from '../../redux';

export const LoadingSplash = ({ navigation }) => {
    const dispatch = useDispatch();
    const [playersLoaded, setPlayersLoaded] = React.useState(false);

    console.log("players loaded:", playersLoaded);
    useEffect(() => {
        if (!playersLoaded) {
// set a timer here too maybe?
            setPlayersLoaded(true);
            dispatch(loadPlayers())
                .then(() => {
                    console.log("load players completed");
                    navigation.reset({ index:0,   routes: [{ name: 'Player Select' }] });
                });
       }
    }, [])

    return (
        <View>
            <Text>
               Loading . . .
            </Text>
        </View>
    );
}
