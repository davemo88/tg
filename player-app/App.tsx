import { StatusBar } from 'expo-status-bar';
import React from 'react';
import { Button, StyleSheet, Text, View } from 'react-native';

export interface PortraitProps {
  name: string;
  address: string;
  picture_url: string;
}

const Portrait: React.FC<PortraitProps> = (props) => {
  return (
    <View>
      <Text>{props.name}</Text>
    </View>
  );
} 

export interface PotProps {
  amount: number;
}

const Pot: React.FC<PotProps> = (props) => {
  return (
    <View>
      <Text>${props.amount}</Text>
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

const Arbiter: React.FC<PortraitProps> = (props) => {
  return (
    <View>
      <Portrait name={props.name} /> 
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
          <Pot amount="100" />
          <PayoutScript script=" winner takes all" />
        </Terms>
        <Players>
          <Portrait name="Akin Toulouse" />
          <Text> vs. </Text>
          <Portrait name="Betsy Wildly" />
        </Players>
        <Arbitration>
          <Arbiter name="Gordon Blue"/>
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
});
