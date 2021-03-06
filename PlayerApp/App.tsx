import 'react-native-gesture-handler';
import React from 'react';
import { Provider } from 'react-redux';
import { NavigationContainer } from '@react-navigation/native';
import { createStackNavigator } from '@react-navigation/stack';

import { store } from './src/redux';

import { LoadingSplash } from './src/components/screens/loading-splash';
import { InitWallet } from './src/components/screens/init-wallet';
import { PlayerSelect } from './src/components/screens/player-select';
import { Home } from './src/components/screens/home';
import { PostContractInfo } from './src/components/screens/post-contract-info';
import { ContractDetails } from './src/components/screens/contract-details';
import { NewPlayer } from './src/components/screens/new-player';
import { AddPlayer } from './src/components/screens/add-player';
import { NewContract } from './src/components/screens/new-contract';
import { ReceiveContract } from './src/components/screens/receive-contract';
import { NewPayout } from './src/components/screens/new-payout';

const Stack = createStackNavigator();

export default function App() {

    return (
      <Provider store={store}>
        <NavigationContainer>
          <Stack.Navigator>
            <Stack.Screen name="Loading Splash" component={LoadingSplash} />
            <Stack.Screen name="Initialize Wallet" component={InitWallet} />
            <Stack.Screen name="Player Select" component={PlayerSelect} />
            <Stack.Screen name="Home" component={Home} />
            <Stack.Screen name="Post Contract Info" component={PostContractInfo} />
            <Stack.Screen name="Contract Details" component={ContractDetails} />
            <Stack.Screen name="New Player" component={NewPlayer} />
            <Stack.Screen name="Add Player" component={AddPlayer} />
            <Stack.Screen name="New Contract" component={NewContract} />
            <Stack.Screen name="Receive Contract" component={ReceiveContract} />
            <Stack.Screen name="New Payout" component={NewPayout} />
          </Stack.Navigator>
        </NavigationContainer>
      </Provider>
    );
}
