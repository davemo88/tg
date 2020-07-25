import { StatusBar } from 'expo-status-bar';
import React from 'react';
import { Image, Button, StyleSheet, Text, TextInput, View } from 'react-native';

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
          style={styles.playerImage}
          source={props.pictureUrl}
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
    <View style={styles.pot}>
      <Text style={{ color:"gold", fontSize:20 }}>Pot</Text>
      <View style={{ alignItems: "center", justifyContent: "center"}}>
        <Image
          style={{ width:154, height:90 }}
          source="http://petcaretips.net/daffyduck2.jpg"
        />
        <TextInput
          onChangeText={text => onChangeText(text)}
          value={value}
          style={{ color: "gold", borderWidth: 1, borderColor: "gold", width: 100, margin: 10, }}
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
    <View style={{ borderWidth: 1}}>
      <Text style={{ padding: 1, fontSize: 16 }}>Arbiter</Text>
      <View style={{ alignItems: "center" }}>
        <Image
          style={styles.arbiterImage}
          source={props.pictureUrl}
        />
      </View>
      <View>
        <Text>{props.name}</Text>
      </View>
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
          <Player name="Akin Toulouse" pictureUrl="https://static-cdn.jtvnw.net/emoticons/v1/425618/1.0"/>
          <View style={{ alignItems: "center", justifyContent: "center" }}>
            <Image
              style={{ width: 25, height: 21, }}
              source="https://i.imgur.com/riXXKnJ.png"
            />
          </View>
          <Player name="Betsy Wildly" pictureUrl="https://static-cdn.jtvnw.net/emoticons/v1/30259/1.0"/>
        </Players>
        <Arbitration>
          <Arbiter name="Gordon Blue" pictureUrl="https://static-cdn.jtvnw.net/emoticons/v1/28/1.0" />
        </Arbitration>
      </View>
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
  },
  terms: {
    flexDirection: "row",
  },
  arbitration: {
  },
  player: {
    padding: 10,
  },
  pot: {
    width: 200, 
    height: 200, 
    borderWidth: 2, 
    padding: 10,
    margin: 2,
  },
  payoutScript: {
    width: 200, 
    height: 200, 
    borderWidth: 2, 
    padding: 10,
    margin: 2,
    //    alignItems: "center", 
    //    justifyContent: "center"
  },
  arbitration: {
    padding: 10,
  },
  playerImage: {
    width: 28,
    height: 28,
  },
  arbiterImage: {
    width: 39,
    height: 27,
  },
});
