import 'react-native-gesture-handler';
import { StatusBar } from 'expo-status-bar';
import React from 'react';
import { Switch, FlatList, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';
import { NavigationContainer } from '@react-navigation/native';
import { createStackNavigator } from '@react-navigation/stack';

export interface PlayerProps {
  name: string;
  address: string;
  pictureUrl: string;
}

const Player: React.FC<PlayerProps> = (props) => {
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

const PlayerSelector = (props) => {
  const [playerIndex, setPlayerIndex] = React.useState(0);
  //  const toggleSwitch = () => setIsEnabled(previousState => !previousState);


  return (
    <View style={styles.playerSelector}>
      <PlayerSelectorButton forward={false} />
        <Player name={props.players[playerIndex].name} pictureUrl={props.players[playerIndex].pictureUrl} />
      <PlayerSelectorButton forward={true} />
    </View>
  );
}

export interface PlayerSelectorButton {
  forward: bool;
}

const PlayerSelectorButton: React.FC<PlayerSelectorButton> = (props) => {

  return (
    <View style={{ justifyContent: 'center', padding: 10 }}>
      <Button
        title={ props.forward ? ">" : "<" } 
        onPress={() => {}}
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
  return(
    <View>
      <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', padding: 5, backgroundColor: 'white', padding: 5, margin: 5, height: 42, }}>
        <View>
          <Text style={{ fontSize: 20, }}>Player</Text>
        </View>
      </View>
      <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', padding: 5, margin: 5, backgroundColor: 'slategrey', }}>
        <Player name="Akin Toulouse" pictureUrl="https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0"/>
        <View style={{ alignItems: 'center' }}>
          <Currency amount='9999' />
          <Text style={{ textDecorationLine: 'underline', color: 'lightblue' }}>Address</Text>
        </View>
      </View> 
    </View> 
  );
}

const ChallengeListItem = (props) => {
  return (
    <View style={{ flexDirection: 'row', justifyContent: 'space-between', backgroundColor: 'slategrey', margin: 5, padding: 5 }}>
      <Player name={props.name} pictureUrl={props.pictureUrl} />
      <View style={{ flexDirection: 'row', padding: 5, margin: 5, alignItems: 'center', justifyContent: 'center', }}>
        <Text>Status . . . </Text>
        <View>
          <View style={{ padding: 20 }}>
            <Currency amount={props.amount} />
          </View>
          <Button 
            title="Details" 
            onPress={() => 
              props.navigation.navigate('Challenge Details')
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

const PlayerSelect = ({ navigation }) => {
  return (
    <View style={styles.newPlayer}>
        <PlayerSelector
          players={[
                {name: "Akin Toulouse", pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0"},
                {name: 'Betsy Wildly', pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0", amount: '100' },
        ]}
        />
        <View style={{ padding: 10 }}>
          <Button 
            title="Ok" 
            onPress={() => 
              navigation.navigate('Home')
            }
          />
        </View>
        <View style={{ padding: 40 }}>
          <Button 
            title="New Player" 
            onPress={() => 
              navigation.navigate('New Player')
            }
          />
        </View>
    </View>
  );
}

const NewPlayer = ({ navigation }) => {
  const [playerName, onChangePlayerName] = React.useState('');
  const [pictureUrl, onChangePictureUrl] = React.useState('');

  return (
    <View style={styles.newPlayer}>
      <Image
        style={styles.mediumEmote}
        source=''
      />
      <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', margin: 10, padding: 10 }}>
        <Text>Player Name</Text>
        <TextInput
          onChangeText={text => onChangePlayerName(text)}
          value={playerName}
          style={{ borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
        />     
      </View>
      <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', margin: 10, padding: 10 }}>
        <Text>Picture Url</Text>
        <TextInput
          onChangeText={text => onChangePictureUrl(text)}
          value={pictureUrl}
          style={{ borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
        />     
      </View>
      <View style={{flexDirection: 'row' }}>
      <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey' }}>
        <Button 
          title="Ok" 
          onPress={() => 
            navigation.navigate('Player Select')
          }
        />
      </View>
      </View>
    </View>
  );
}

const Home = ({ navigation }) => {
  return (
    <View style={styles.home}>
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
              data={[
                {name: 'Betsy Wildly', pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0", amount: '100' },
                {name: 'Betsy Wildly', pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0", amount: '200' },
              ]}
              renderItem={({item}) => <ChallengeListItem navigation={navigation} name={item.name} pictureUrl={item.pictureUrl} amount={item.amount} />}
            />
          </View>
        </View>
    </View>
  );
}

const NewChallenge = ({ navigation }) => {
  const [playerName, onChangePlayerName] = React.useState('');
  const [challengeAmount, onChangeChallengeAmount] = React.useState('');

  return (
    <View style={styles.newPlayer}>
      <Text style={{ fontSize: 20 }}>Choose Opponent</Text>
      <PlayerSelector
        players={[
              {name: 'Betsy Wildly', pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0", amount: '100' },
              {name: 'Betsy Wildly', pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0", amount: '200' },
      ]}
      />
      <View style={{ margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
        <Button 
          title="New Opponent" 
          onPress={() => navigation.navigate('New Opponent') }
        />
      </View>
      <View style={{ backgroundColor: 'lightslategrey', alignItems: 'center', padding: 10 }}>
        <Text style={{ fontSize: 16 }}>Amount</Text>
        <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', }}>
          <TextInput
            onChangeText={text => onChangeChallengeAmount(text)}
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
            title="Send" 
            onPress={() => navigation.navigate('Home') }
          />
        </View>
      </View>
    </View>
  );
}

const ChallengeDetails = ({ navigation }) => {
  return (
    <View style={styles.challengeDetails}>
      <View style={{ flex: 2, alignItems: 'center', justifyContent: 'space-around', }}>
        <View style= {{flexDirection: 'row', justifyContent: 'space-between' }}>
          <View style={{ flex: 1 }}>
            <Text style={{ fontSize: 20 }}>Opponent</Text>
            <Player name='Betsy Wildly' pictureUrl='https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0' />
          </View>
          <View style={{ flex: 1, alignItems: 'flex-end' }}>
              <Text style={{ fontSize: 20 }}>Amount</Text>
            <View style={{ flex: 1, justifyContent: 'center' }}>
              <Currency amount='100' />
            </View>
          </View>
        </View>
        <View>
          <Text style={{ fontSize: 20 }}>Status</Text>
          <Text> . . . </Text>
        </View>
        <View>
          <Text style={{ fontSize: 20 }}>Arbiter</Text>
          <Arbiter name='Gordon Blue' pictureUrl='https://static-cdn.jtvnw.net/emoticons/v1/28/1.0' />
        </View>
      </View>
      <View style={{ flex: 1,  }}>
        <View style={{ margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
          <Button 
            title="Players Payout" 
            onPress={() => navigation.navigate('Players Payout') }
          />
        </View>
        <View style={{ margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
          <Button 
            title="Arbiter Payout" 
            onPress={() => navigation.navigate('Arbiter Payout') }
          />
        </View>
      </View>
    </View>
  );
}

const ArbiterPayout = ({ navigation }) => {
  const [refToken, onChangeRefToken] = React.useState('');

  return (
    <View style={styles.payoutRequest}>
      <View style={{ alignItems: 'center' }}>
        <View>
          <Text style={{ fontSize: 20 }}>Recipient</Text>
        </View>
        <PlayerSelector
          players={[
                {name: "Akin Toulouse", pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0"},
                {name: 'Betsy Wildly', pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0", amount: '200' },
        ]}
        />
        <Currency amount='100' />
      </View>
      <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', margin: 10, padding: 10 }}>
        <Text>Referee Token</Text>
        <TextInput
          onChangeText={text => onChangeRefToken(text)}
          value={refToken}
          style={{ borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
        />     
      </View>
      <View>
        <Text style={{ fontSize: 16 }}>Arbiter</Text>
        <Arbiter name='Gordon Blue' pictureUrl='https://static-cdn.jtvnw.net/emoticons/v1/28/1.0' />
      </View>
      <SignatureSwitch />
      <View style={{ margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
        <Button 
          title="Send" 
          onPress={() => navigation.navigate('Home') }
        />
      </View>
    </View>
  );
}

const PlayersPayout = ({ navigation }) => {
  return (
    <View style={styles.payoutRequest}>
      <View>
        <View style={{ alignItems: 'center' }}>
          <Text style={{ fontSize: 20 }}>Recipient</Text>
        </View>
        <PlayerSelector
          players={[
                {name: 'Betsy Wildly', pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0", amount: '100' },
                {name: 'Betsy Wildly', pictureUrl: "https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0", amount: '200' },
        ]}
        />
      </View>
      <Currency amount='100' />
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

const NewOpponent = ({ navigation }) => {
  const [opponentName, onChangeOpponentName] = React.useState('');

  return (
    <View style={styles.newPlayer}>
      <Image
        style={styles.mediumEmote}
        source=''
      />
      <View style={{alignItems: 'center', backgroundColor: 'lightslategrey', margin: 10, padding: 10 }}>
        <TextInput
          onChangeText={text => onChangeOpponentName(text)}
          value={opponentName}
          style={{ borderWidth: 1, flex: 1, margin: 10, padding: 4, }}
        />     
        <Text>Enter Opponent Name or Address</Text>
      </View>
      <View style={{flexDirection: 'row' }}>
        <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey' }}>
          <Button 
            title="Ok" 
            onPress={() => navigation.navigate('Player Select')}
          />
       </View>
     </View>
   </View>
  );
}

const Stack = createStackNavigator();

export default function App() {
  return (
    <NavigationContainer>
      <Stack.Navigator>
        <Stack.Screen name="New Opponent" component={NewOpponent} />
        <Stack.Screen name="Player Select" component={PlayerSelect} />
        <Stack.Screen name="Home" component={Home} />
        <Stack.Screen name="Challenge Details" component={ChallengeDetails} />
        <Stack.Screen name="New Player" component={NewPlayer} />
        <Stack.Screen name="New Challenge" component={NewChallenge} />
        <Stack.Screen name="Players Payout" component={PlayersPayout} />
        <Stack.Screen name="Arbiter Payout" component={ArbiterPayout} />
      </Stack.Navigator>
    </NavigationContainer>
  );
}

const styles = StyleSheet.create({
  players: {
    flexDirection: 'row',
    flex: 2,
  },
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
    alignItems: 'stretch',
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
