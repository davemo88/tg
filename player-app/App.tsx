import { StatusBar } from 'expo-status-bar';
import React from 'react';
import { Image, Button, StyleSheet, Text, TextInput, View } from 'react-native';

export interface PlayerProps {
  name: string;
  address: string;
  picture_url: string;
}

const Player: React.FC<PlayerProps> = (props) => {
  return (
    <View>
      <View>
        <Image
          style={styles.player_image}
          source={props.picture_url}
        />
      </View>
      <View>
        <Text>{props.name}</Text>
      </View>
    </View>
  );
} 

export interface PotProps {
  amount: number;
}

const Pot: React.FC<PotProps> = (props) => {
  const [value, onChangeText] = React.useState('0');

  return (
    <View>
      <TextInput
        style={{ height: 40, borderColor: 'grey', borderWidth: 1 }}
        onChangeText={text => onChangeText(text)}
        value={value}
      />     
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

const Players = (props) => {
  return (
    <View style={styles.players}>
      {props.children}
    </View>
  );
} 

const Arbitration = (props) => {
  return (
    <View style={styles.arbitration}>
      {props.children}
    </View>
  );
}

const Arbiter: React.FC<PlayerProps> = (props) => {
  return (
    <View>
      <View>
        <Image
          style={styles.arbiter_image}
          source={props.picture_url}
        />
      </View>
      <View>
        <Text>{props.name}</Text>
      </View>
      <Text>arbiter</Text>
    </View>
  );
} 

export interface PayoutScriptProps {
  script: string;
}

const PayoutScript: React.FC<PayoutScriptProps> = (props) => {
  return (
    <View>
      <Text>{props.script}</Text>
    </View>
  );
}
  

export default function App() {
  return (
      <View style={styles.container}>
        <Terms>
          <Pot />
          <PayoutScript script=" winner takes all" />
        </Terms>
        <Players>
          <Player name="Akin Toulouse" picture_url="https://static-cdn.jtvnw.net/emoticons/v1/425618/1.0"/>
          <Text> vs. </Text>
          <Player name="Betsy Wildly" picture_url="https://static-cdn.jtvnw.net/emoticons/v1/30259/1.0"/>
        </Players>
        <Arbitration>
          <Arbiter name="Gordon Blue" picture_url="https://static-cdn.jtvnw.net/emoticons/v1/28/1.0" />
        </Arbitration>
      </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
    alignItems: 'center',
    justifyContent: 'center',
  },
  players: {
    flexDirection: "row",
  },
  terms: {
    flexDirection: "row",
  },
  arbitration: {
  },
  player_image: {
    width: 28,
    height: 28,
  },
  arbiter_image: {
    width: 39,
    height: 27,
  },
});
