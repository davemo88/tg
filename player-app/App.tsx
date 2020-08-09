import 'react-native-gesture-handler';
import { StatusBar } from 'expo-status-bar';
import React from 'react';
import { useSelector, useDispatch, Provider } from 'react-redux';
import { nanoid } from '@reduxjs/toolkit'
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';
import { NavigationContainer } from '@react-navigation/native';
import { createStackNavigator } from '@react-navigation/stack';
import Slider from '@react-native-community/slider';


import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, challengeSelectors, challengeSlice, selectedLocalPlayerIdSlice, } from './src/redux.ts';
import { Player } from './src/datatypes.ts'

export interface PlayerPortraitProps {
  name: string;
  pictureUrl: string;
}

const PlayerPortrait: React.FC<PlayerPortraitProps> = (props) => {
  return (
    <View style={styles.player}>
      <View style={{ alignItems: "center" }}>
        <Image
          style={styles.mediumEmote}
          source={props.pictureUrl}
        />
      </View>
      <View style={{ alignItems: 'center', backgroundColor: "slategrey", padding: 1 }}>
        <Text style={{ fontSize: 17 }}>{props.name}</Text>
      </View>
    </View>
  );
} 

export interface PlayerSelectorProps {
  playerIds: string[];
  selectedPlayerId: string;
  setSelectedPlayerId: (newPlayerId: string) => void;
}

const PlayerSelector = (props) => {
  const selectedPlayer = playerSelectors.selectById(store.getState(), props.selectedPlayerId);

  return (
    <View style={styles.playerSelector}>
      <PlayerSelectorButton forward={false} selectedPlayerId={props.selectedPlayerId} setSelectedPlayerId={props.setSelectedPlayerId} playerIds={props.playerIds} />
      <PlayerPortrait 
        name={selectedPlayer.name}
        pictureUrl={selectedPlayer.pictureUrl}
      />
      <PlayerSelectorButton forward={true} selectedPlayerId={props.selectedPlayerId} setSelectedPlayerId={props.setSelectedPlayerId} playerIds={props.playerIds} />
    </View>
  );
}

export interface PlayerSelectorButton {
  forward: bool;
  playerIds: string[];
  selectedPlayerId: string;
  setSelectedPlayerId: (newPlayerId: string) => void;
}

const PlayerSelectorButton = (props) => {
  const playerIndex = props.playerIds.findIndex((playerId) => playerId === props.selectedPlayerId );

  return (
    <View style={{ justifyContent: 'center', padding: 10 }}>
      <Button
        title={ props.forward ? ">" : "<" } 
        onPress={() => {
          let newPlayerIndex = props.forward ? playerIndex+1 : playerIndex-1;
          newPlayerIndex = (newPlayerIndex + props.playerIds.length) % props.playerIds.length;
          props.setSelectedPlayerId(props.playerIds[newPlayerIndex]);
        }}
      />
    </View>
  );
}

const Currency = (props) => {
  return (
    <View style={{ flexDirection: 'row', alignItems: 'center', }}>
      <Text style={{ fontSize: 16 }}>{props.amount}</Text>
      <CurrencySymbol />
    </View>
  );
}

const CurrencySymbol = (props) => {
  return (
    <Image
      style={styles.smallEmote}
      source="https://static-cdn.jtvnw.net/emoticons/v1/90076/1.0"
    />
  );
}

const SignatureSwitch = (props) => {
  const [isEnabled, setIsEnabled] = React.useState(false);
  const toggleSwitch = () => setIsEnabled(previousState => !previousState);

  return (
    <View style={{ flexDirection: 'row', backgroundColor: 'lightslategrey', alignItems: 'center', justifyContent: 'space-between', padding: 10, margin: 10, }}>
      <View style={{ flex: 1 }}>
        <Text style={{ fontSize: 16, }}>Sign</Text>
      </View>
      <View style={{ flex: 1 }}>
        <Switch 
          onValueChange={toggleSwitch}
          value={isEnabled}
        />
      </View>
    </View>
  );
}

const HomeHeader = (props) => {
  const selectedLocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);
  const selectedPlayer = playerSelectors.selectById(store.getState(), selectedLocalPlayer.playerId);

  return(
    <View>
      <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', padding: 5, backgroundColor: 'white', padding: 5, margin: 5, height: 42, }}>
        <View>
          <Text style={{ fontSize: 20, }}>Player</Text>
        </View>
      </View>
      <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', padding: 5, margin: 5, backgroundColor: 'slategrey', }}>
        <PlayerPortrait name={selectedPlayer.name} pictureUrl={selectedPlayer.pictureUrl} />
        <View style={{ alignItems: 'center' }}>
          <Currency amount={selectedLocalPlayer.balance} />
          <Text style={{ textDecorationLine: 'underline', color: 'lightblue' }}>Address</Text>
        </View>
      </View> 
    </View> 
  );
}

const ChallengeListItem = (props) => {
  const playerTwo = playerSelectors.selectById(store.getState(), props.challenge.playerTwoId);

  return (
    <View style={{ flexDirection: 'row', justifyContent: 'space-between', backgroundColor: 'slategrey', margin: 5, padding: 5 }}>
      <PlayerPortrait name={playerTwo.name} pictureUrl={playerTwo.pictureUrl} />
      <View style={{ flexDirection: 'row', padding: 5, margin: 5, alignItems: 'center', justifyContent: 'center', }}>
        <Text>Status: {props.challenge.status}</Text>
        <View>
          <View style={{ padding: 20 }}>
            <Currency amount={props.challenge.pot} />
          </View>
          <Button 
            title="Details" 
            onPress={() => 
              props.navigation.push('Challenge Details', { challengeId: props.challenge.id })
            }
          />
        </View>
      </View>
    </View>
  );
}

const Arbiter: React.FC<PlayerProps> = (props) => {
  return (
    <View style={styles.arbiter}>
      <View style={{ alignItems: "center", padding: 2, margin: 2, }}>
        <Image
          style={styles.arbiterImage}
          source={props.pictureUrl}
        />
      </View>
      <View style={{ backgroundColor: "slategrey", padding: 1, flexDirection: "row" }}>
        <Image
          style={{ height: 17, width: 17 }}
          source="https://static-cdn.jtvnw.net/emoticons/v1/156787/1.0"
        />
        <Text>{props.name}</Text>
      </View>
    </View>
  );
} 

const LocalPlayerSelect = ({ navigation }) => {
  const localPlayers = localPlayerSelectors.selectAll(store.getState());
  const [selectedPlayerId, setSelectedPlayerId] = React.useState(localPlayers[0].playerId)

  return (
    <View style={styles.newPlayer}>
      <PlayerSelector 
        selectedPlayerId={selectedPlayerId}
        setSelectedPlayerId={setSelectedPlayerId}
        playerIds={localPlayers.map(l => l.playerId)}
      />
      <View style={{ padding: 10 }}>
        <Button 
          title="Ok" 
          onPress={() => {
            const selectedLocalPlayer = localPlayers.find(l => l.playerId === selectedPlayerId);
            store.dispatch(selectedLocalPlayerIdSlice.actions.setSelectedLocalPlayerId(selectedLocalPlayer.id));
            navigation.navigate('Home')
          } }
        />
      </View>
      <View style={{ padding: 40 }}>
        <Button 
          title="New Local Player" 
          onPress={() => {
            navigation.navigate('New Local Player')
          } }
        />
      </View>
    </View>
  );
}

const NewLocalPlayer = ({ navigation }) => {
  const [playerName, setPlayerName] = React.useState('');
  const [pictureUrl, setPictureUrl] = React.useState('');

  return (
    <View style={styles.newPlayer}>
      <PlayerPortrait name={playerName} pictureUrl={pictureUrl} />
      <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', margin: 10, padding: 10 }}>
        <Text>Player Name</Text>
        <TextInput
          onChangeText={text => setPlayerName(text)}
          value={playerName}
          style={{ borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
        />     
      </View>
      <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', margin: 10, padding: 10 }}>
        <Text>Picture Url</Text>
        <TextInput
          onChangeText={text => setPictureUrl(text)}
          value={pictureUrl}
          style={{ borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
        />     
      </View>
      <View style={{flexDirection: 'row' }}>
      <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey' }}>
        <Button 
          title="Ok" 
          onPress={() => {
            const newPlayerId = nanoid();
            store.dispatch(playerSlice.actions.playerAdded({ id: newPlayerId, name: playerName, pictureUrl: pictureUrl }));
            store.dispatch(localPlayerSlice.actions.localPlayerAdded({ id: nanoid(), playerId: newPlayerId, balance: 0 }));
            navigation.push('Player Select')
          } }
        />
      </View>
      </View>
    </View>
  );
}

const Home = ({ navigation }) => {
  const selectedLocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);

  const challenges = challengeSelectors
    .selectAll(store.getState())
    .filter((challenge, i, a) =>{ return challenge.playerOneId === selectedLocalPlayer.playerId });

  return (
    <View style={styles.home}>
      <View style={{ minWidth: 360, flex:1, alignItems: 'stretch' }}>
        <View style={{ flex: 1, justifyContent: 'flex-start', }}>
          <HomeHeader />
        </View>
        <View style={{ flex: 3, }}>
          <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', padding: 5, backgroundColor: 'white', padding: 5, margin: 5, height: 42, }}>
            <View>
              <Text style={{ fontSize: 20, }}>Challenges</Text>
            </View>
            <View>
              <Button 
                title="New"
                onPress={() => 
                  navigation.navigate('New Challenge')
                }
              />
            </View>
          </View>
          <View style={{ padding: 5, }}>
            <FlatList
              data={challenges}
              renderItem={({item}) => <ChallengeListItem navigation={navigation} challenge={item} />}
            />
          </View>
        </View>
      </View>
    </View>
  );
}

const NewChallenge = ({ navigation }) => {
  const selectedLocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);
  const playerTwos = playerSelectors
    .selectAll(store.getState())
    .filter((player, i, a) => player.id != selectedLocalPlayer.playerId);
  const [challengeAmount, onChangeChallengeAmount] = React.useState('0');
  const [playerTwoId, setPlayerTwoId] = React.useState(playerTwos[0].id);
  const [isSigned, setIsSigned] = React.useState(false);

  const valid = () => {
    if ((parseInt(challengeAmount) > 0) && isSigned) {
      return true
    }
    return false
  }

  return (
    <View style={styles.newPlayer}>
      <Text style={{ fontSize: 20 }}>Choose Player</Text>
      <PlayerSelector selectedPlayerId={playerTwoId} setSelectedPlayerId={setPlayerTwoId} playerIds={playerTwos.map(p => p.id)} />
      <View style={{ margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
        <Button 
          title="New Player" 
          onPress={() => navigation.navigate('New Player') }
        />
      </View>
      <View style={{ backgroundColor: 'lightslategrey', alignItems: 'center', padding: 10 }}>
        <Text style={{ fontSize: 16 }}>Amount</Text>
        <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', }}>
          <TextInput
            onChangeText={text => onChangeChallengeAmount(text)}
            onBlur={() => {if (Number.isNaN(parseInt(challengeAmount))) { onChangeChallengeAmount('0')}}}
            value={challengeAmount}
            style={{ borderWidth: 1, width: 100, margin: 10, padding: 4, textAlign: 'right' }}
          />     
          <CurrencySymbol />
        </View>
      </View>
      <View style={{ flexDirection: 'row' }}>
        <SignatureSwitch />
        <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
          <Button 
            disabled={!valid()}
            title="Issue" 
            onPress={() => {
              store.dispatch(challengeSlice.actions.challengeAdded({ 
                id: nanoid(),
                playerOneId: selectedLocalPlayer.playerId,
                playerTwoId: playerTwoId,
                pot: challengeAmount,
                status: 'Issued',
              }))
              navigation.push('Home') 
            } }
          />
        </View>
      </View>
    </View>
  );
}

const NewPlayer = ({ navigation }) => {
  const [playerName, setPlayerName] = React.useState('');

  return (
    <View style={styles.newPlayer}>
      <Image
        style={styles.mediumEmote}
        source=''
      />
      <View style={{alignItems: 'center', backgroundColor: 'lightslategrey', margin: 10, padding: 10 }}>
        <TextInput
          onChangeText={text => setPlayerName(text)}
          value={playerName}
          style={{ borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
        />     
        <Text>Enter Player Name or Address</Text>
      </View>
      <View style={{flexDirection: 'row' }}>
        <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey' }}>
          <Button 
            title="Ok" 
            onPress={() => {
              const newPlayerId = nanoid();
              store.dispatch(playerSlice.actions.playerAdded({ id: newPlayerId, name: playerName, pictureUrl: 'https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0' }));
              setPlayerName('');
              navigation.push('New Challenge')
            } }
          />
       </View>
     </View>
   </View>
  );
}

const ChallengeDetails = ({ route, navigation }) => {
  const { challengeId } = route.params;
  const challenge = challengeSelectors.selectById(store.getState(), challengeId);
  const playerTwo = playerSelectors.selectById(store.getState(), challenge.playerTwoId);
  return (
    <View style={styles.challengeDetails}>
      <View style={{ flex: 2, alignItems: 'center', justifyContent: 'space-around', }}>
        <View style= {{flexDirection: 'row', justifyContent: 'space-between' }}>
          <View style={{ flex: 1 }}>
            <Text style={{ fontSize: 20 }}>Player 2</Text>
            <PlayerPortrait name={playerTwo.name} pictureUrl={playerTwo.pictureUrl} />
          </View>
          <View style={{ flex: 1, alignItems: 'flex-end' }}>
              <Text style={{ fontSize: 20 }}>Pot</Text>
            <View style={{ flex: 1, justifyContent: 'center' }}>
              <Currency amount={challenge.pot} />
            </View>
          </View>
        </View>
        <View>
          <Text style={{ fontSize: 20 }}>Status</Text>
          <Text>{challenge.status}</Text>
        </View>
        <View>
          <Text style={{ fontSize: 20 }}>Arbiter</Text>
          <Arbiter name='Gordon Blue' pictureUrl='https://static-cdn.jtvnw.net/emoticons/v1/28/1.0' />
        </View>
      </View>
      <View style={{ flex: 1,  }}>
        <View style={{ margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
          <Button 
            title="Request Payout" 
            onPress={() => navigation.push('Request Payout', { challengeId }) }
          />
        </View>
      </View>
    </View>
  );
}

const RequestPayout = ({ route, navigation }) => {
  const { challengeId } = route.params;
  const challenge = challengeSelectors.selectById(store.getState(), challengeId);
  const playerTwo = playerSelectors.selectById(store.getState(), challenge.playerTwoId)
  const selectedLocalPlayer = localPlayerSelectors.selectById(store.getState(), store.getState().selectedLocalPlayerId);
  const selectedPlayer = playerSelectors.selectById(store.getState(), selectedLocalPlayer.playerId);
  const [playerOnePayout, setPlayerOnPayout] = React.useState(challenge.pot);
  const [playerTwoPayout, setPlayerTwoPayout] = React.useState(0);
  const [isArbitratedPayout, setIsArbitratedPayout] = React.useState(false);
  const toggleArbitration = () => setIsArbitratedPayout(previousState => !previousState);
  const [arbitrationToken, setArbitrationToken] = React.useState('');

  return (
    <View style={styles.payoutRequest}>
      <View>
        <View style={{ alignItems: 'center' }}>
          <View style={{ flexDirection: 'row', alignItems: 'center' }}>
            <Text style={{ fontSize: 16 }}>Total Pot: </Text>
            <Currency amount={challenge.pot} />
          </View>
          <Text style={{ fontSize: 16 }}>Distribute Pot</Text>
          <View style={{ flexDirection: 'row', }}>
          </View>
          <View style={{ flexDirection: 'row', justifyContent: 'flex-start', padding: 10 }}>
            <View style={{ alignItems: 'center', padding: 5 }}>
              <PlayerPortrait name={selectedPlayer.name} pictureUrl={selectedPlayer.pictureUrl} />
              <Currency amount={ localPlayerPayout } />
            </View>
            <View style={{ alignItems: 'center', padding: 5 }}>
              <PlayerPortrait name={playerTwo.name} pictureUrl={playerTwo.pictureUrl} />
              <Currency amount={ playerTwoPayout } />
            </View>
          </View>
          <Slider
            style={{ width: 200, height: 40, padding: 5, margin: 5, }}
            value="0"
            onValueChange={ (value) => {
              const newPLayerOnePayout = Math.floor((1-value) * challenge.pot);
              setPlayerOnePayout(newPLayerOnePayout);
              setPlayerTwoPayout(challenge.pot - newPLayerOnePayout);
            }}
          />
        </View>
      </View>
      <View style={{ padding: 5 }}>
        <Text>Arbitrated Payout</Text>
        <Switch 
          onValueChange={toggleArbitration}
          value={isArbitratedPayout}
        />
      </View>
      { isArbitratedPayout && 
        <View style={{ alignItems: 'center' }}>
          <Text style={{ padding: 2 }}>Arbiter</Text>
          <View>
            <Arbiter name='Gordon Blue' pictureUrl='https://static-cdn.jtvnw.net/emoticons/v1/28/1.0' />
          </View>
          <Text style={{ padding: 2 }}>Token</Text>
          <TextInput
            value={arbitrationToken}
            onChangeText={text => setArbitrationToken(text)}
            style={{ borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
          />
        </View>
      }
      <View style={{ flexDirection: 'row' }}>
        <SignatureSwitch />
        <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
          <Button 
            title="Send" 
            onPress={() => navigation.navigate('Home') }
          />
        </View>
      </View>
    </View>
  )
}

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
          <Stack.Screen name="New Player" component={NewPlayer} />
          <Stack.Screen name="New Challenge" component={NewChallenge} />
          <Stack.Screen name="Request Payout" component={RequestPayout} />
        </Stack.Navigator>
      </NavigationContainer>
    </Provider>
  );
}

const styles = StyleSheet.create({
  terms: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  arbitration: {
    borderWidth: 1,
  },
  arbiter: {
    backgroundColor: 'lightslategrey',
    padding: 10,
    borderWidth: 1,
  },
  player: {
    padding: 10,
    minWidth: 140,
    backgroundColor: 'lightslategrey',
  },
  payoutScript: {
    flex: 1,
    width: 150, 
    height: 150, 
    borderWidth: 2, 
    padding: 5,
  },
  pot: {
    flex: 1,
    padding: 5,
    margin: 30,
    alignItems: 'center',
    justifyContent: 'center',
  },
  arbitration: {
    margin: 10,
  },
  smallEmote: {
    width: 28,
    height: 28,
  },
  mediumEmote: {
    width: 56,
    height: 56,
  },
  arbiterImage: {
    width: 39,
    height: 27,
  },
  playerSelect: {
    flex: 1,
    backgroundColor: 'grey',
    alignItems: 'center',
    justifyContent: 'center',
  },
  playerSelector: {
    padding: 10,
    margin: 10,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
  },
  newPlayer: {
    flex: 1,
    backgroundColor: 'grey',
    alignItems: 'center',
    justifyContent: 'center',
  },
  home: {
    flex: 1,
    backgroundColor: 'grey',
    alignItems: 'center',
    justifyContent: 'flex-start',
  },
  challengeDetails: {
    flex: 1,
    backgroundColor: 'grey',
    alignItems: 'center',
    justifyContent: 'center',
  },
  payoutRequest: {
    flex: 1,
    backgroundColor: 'grey',
    alignItems: 'center',
    justifyContent: 'center',
  },
});
