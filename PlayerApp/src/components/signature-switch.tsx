import React from 'react';
import { Switch, Text, View, } from 'react-native';

import { styles } from '../styles';

export interface SignatureSwitchProps {
  isSigned: boolean;
  setIsSigned: (newIsSigned: boolean) => void;
}

export const SignatureSwitch: React.FC<SignatureSwitchProps> = (props) => {
  const toggleSwitch = () => props.setIsSigned(previousState => !previousState);

  return (
    <View style={{ flex: 1, flexDirection: 'row', backgroundColor: 'lightslategrey', alignItems: 'center', justifyContent: 'space-between', padding: 10, margin: 10, }}>
      <View style={{ flex: 1 }}>
        <Text style={{ fontSize: 16, }}>Sign</Text>
      </View>
      <View style={{ flex: 1 }}>
        <Switch 
          onValueChange={toggleSwitch}
          value={props.isSigned}
        />
      </View>
    </View>
  );
}
