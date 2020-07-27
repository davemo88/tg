import 'react-native-gesture-handler';
import { StatusBar } from 'expo-status-bar';
import React from 'react';
import { Image, Button, StyleSheet, Text, TextInput, View } from 'react-native';
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
      <View style={{ backgroundColor: "slategrey", padding: 1 }}>
        <Text style={{ fontSize: 17 }}>{props.name}</Text>
      </View>
    </View>
  );
} 

const Players = (props) => {
  return (
    <View>
      <View style={{ alignItems: "center", padding: 5 }}>
        <Text style={{ fontSize: 20 }}>Players</Text>
      </View>
      <View style={styles.players}>
        {props.children}
      </View>
    </View>
  );
} 


export interface PotProps {
  amount: number;
}

const Pot: React.FC<PotProps> = (props) => {
  const [value, onChangeText] = React.useState('100');

  return (
    <View style={styles.pot}>
      <Text style={{ color:"gold", fontSize:20,}}>Pot</Text>
      <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "center"}}>
        <TextInput
          onChangeText={text => onChangeText(text)}
          value={value}
          style={{ borderWidth: 1, borderColor: "gold", width: 75, margin: 10, textAlign: "right", padding: 4, }}
        />     
        <Image
          style={styles.smallEmote}
          source="https://static-cdn.jtvnw.net/emoticons/v1/90076/1.0"
        />
      </View>
    </View>
  );
} 

export interface PayoutScriptProps {
  script: string;
}

const PayoutScript: React.FC<PayoutScriptProps> = (props) => {
  return (
    <View style={styles.payoutScript}>
      <Text style={{ color:"lime", fontSize:20 }}>Payouts</Text>
      <View style={{ borderColor: "lime", borderWidth: 1, flex: 1 }}>
        <Text style={{ padding: 5 }}>{props.script}</Text>
      </View>
    </View>
  );
}

const Terms = (props) => {
  return (
    <View style={styles.terms}>
      {props.children}
    </View>
  );
}

const Arbitration = (props) => {
  return (
    <View style={styles.arbitration}>
      <View style={{ alignItems: "center" }}>
        <Text style={{ fontSize: 15 }}>Arbiter</Text>
      </View>
      <View >
        {props.children}
      </View>
    </View>
  );
}

const Arbiter: React.FC<PlayerProps> = (props) => {
  return (
    <View style={styles.arbiter}>
      <View style={{ alignItems: "center" }}>
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

const OldChallenge = (props) => {
  return (
      <View style={styles.container}>
        <Text style={{ margin: 10, fontSize: 24 }}>New Challenge</Text>
        <Terms>
          <Pot />
        </Terms>
        <Players>
          <Player name="Akin Toulouse" pictureUrl="https://static-cdn.jtvnw.net/emoticons/v1/425618/2.0"/>
          <View style={{ alignItems: "center", justifyContent: "center", margin: 5 }}>
            <Image
              style={{ width: 25, height: 21, }}
              source="https://i.imgur.com/riXXKnJ.png"
            />
          </View>
          <Player name="Betsy Wildly" pictureUrl="https://static-cdn.jtvnw.net/emoticons/v1/30259/2.0"/>
        </Players>
        <Arbitration>
          <Arbiter name="Gordon Blue" pictureUrl="https://static-cdn.jtvnw.net/emoticons/v1/28/1.0" />
        </Arbitration>
      </View>
  );
}

const PlayerSelect = (props) => {
  return (
    <View style={styles.playerSelect}>
    </View>
  );
}

const NewPlayer = (props) => {
  return (
    <View style={styles.newPlayer}>
    </View>
  );
}

const Home = (props) => {
  return (
    <View style={styles.home}>
    </View>
  );
}

const NewChallenge = (props) => {
  return (
    <View style={styles.newChallenge}>
    </View>
  );
}

const ChallengeDetails = (props) => {
  return (
    <View style={styles.challengeDetails}>
    </View>
  );
}

const PayoutRequest = (props) => {
  return (
    <View style={styles.payoutRequest}>
    </View>
  );
}

const Stack = createStackNavigator();

export default function App() {
  return (
    <NavigationContainer>
      <Stack.Navigator>
        <Stack.Screen name="Old Challenge" component={OldChallenge} />
        <Stack.Screen name="Player Select" component={PlayerSelect} />
        <Stack.Screen name="New Player" component={NewPlayer} />
        <Stack.Screen name="Home" component={Home} />
        <Stack.Screen name="New Challenge" component={NewChallenge} />
        <Stack.Screen name="Challenge Details" component={ChallengeDetails} />
        <Stack.Screen name="Payout Request" component={PayoutRequest} />
      </Stack.Navigator>
    </NavigationContainer>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: 'grey',
    alignItems: 'center',
    justifyContent: 'center',
  },
  players: {
    flexDirection: "row",
    flex: 2,
  },
  terms: {
    flexDirection: "row",
    alignItems: 'center',
  },
  arbitration: {
    borderWidth: 1,
  },
  arbiter: {
    backgroundColor: "lightslategrey",
    padding: 10,
    margin: 5,
    borderWidth: 1,
  },
  player: {
    padding: 10,
    backgroundColor: "lightslategrey",
    borderWidth: 1,
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
  potImage: {
    width: 56,
    height: 56,
  },
  arbiterImage: {
    width: 39,
    height: 27,
  },
});
