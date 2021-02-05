import 'react-native-gesture-handler';
import React, { useEffect } from 'react';
import { useDispatch } from 'react-redux';
import { Text, View, } from 'react-native';

import { store, loadPlayers } from '../../redux';

export const LoadingSplash = ({ navigation }) => {
//    const [playersLoading, setPlayersLoading] = React.useState(false);
    const dispatch = useDispatch();

    console.log("render loading splash");
//    console.log("players loading:", playersLoading);
    useEffect(() => {
        console.log("using effect");
//        if (!playersLoading) {
//            setPlayersLoading(true);
            dispatch(loadPlayers)
                .then(() => {
                    console.log("load players completed");
                    navigation.reset({ index:0, routes: [{ name: 'Player Select' }] });
                });
//        }
    }, []);

    return (
        <View>
            <Text>
               Loading . . .
            </Text>
        </View>
    );
}
