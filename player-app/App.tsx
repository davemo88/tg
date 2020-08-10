import 'react-native-gesture-handler';
import React from 'react';
import { Provider } from 'react-redux';
import { NavigationContainer } from '@react-navigation/native';
import { createStackNavigator } from '@react-navigation/stack';

import { store } from './src/redux.ts';

import { LocalPlayerSelect } from './src/components/screens/local-player-select.tsx';
import { Home } from './src/components/screens/home.tsx';
import { ChallengeDetails } from './src/components/screens/challenge-details.tsx';
import { NewLocalPlayer } from './src/components/screens/new-local-player.tsx';
import { AddPlayer } from './src/components/screens/add-player.tsx';
import { NewChallenge } from './src/components/screens/new-challenge.tsx';
import { RequestPayout } from './src/components/screens/request-payout.tsx';

const Stack = createStackNavigator();

export default function App() {
  return (
    <Provider store={store}>
      <NavigationContainer>
        <Stack.Navigator>
          <Stack.Screen name="Player Select" component={LocalPlayerSelect} />
          <Stack.Screen name="Home" component={Home} />
          <Stack.Screen name="Challenge Details" component={ChallengeDetails} />
          <Stack.Screen name="New Local Player" component={NewLocalPlayer} />
          <Stack.Screen name="Add Player" component={AddPlayer} />
          <Stack.Screen name="New Challenge" component={NewChallenge} />
          <Stack.Screen name="Request Payout" component={RequestPayout} />
        </Stack.Navigator>
      </NavigationContainer>
    </Provider>
  );
}
