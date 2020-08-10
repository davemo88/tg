import React from 'react';
import {Image, Text, View, } from 'react-native';

import { styles } from '../styles.ts';

export const Arbiter = (props) => {
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

