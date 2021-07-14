import React from 'react';
import { Text, Image, View, } from 'react-native';

import { styles } from '../styles.ts';

type CurrencyProps = {
    amount: number,
}

export const Currency = (props: CurrencyProps) => {
  return (
    <View style={{ flexDirection: 'row', alignItems: 'center', }}>
      <Text style={{ fontSize: 16 }}>{props.amount}</Text>
      <CurrencySymbol />
    </View>
  );
}

const CurrencySymbol = () => {
  return (
    <Image
      style={styles.smallEmote}
      source={{uri: "https://static-cdn.jtvnw.net/emoticons/v1/90076/1.0"}}
    />
  );
}

