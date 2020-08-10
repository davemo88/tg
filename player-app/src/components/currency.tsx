import React from 'react';
import { Text, Image, View, } from 'react-native';

import { styles } from '../styles.ts';

export const Currency = (props) => {
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

