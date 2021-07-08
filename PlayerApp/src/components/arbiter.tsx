import React from 'react';
import {Image, Text, View, } from 'react-native';

import { styles } from '../styles.ts';

export const ARBITER_NAME = 'Gordon Blue';
export const ARBITER_PICTURE_URL = 'https://static-cdn.jtvnw.net/emoticons/v1/28/1.0';

export const Arbiter = (props) => {
  return (
    <View style={styles.arbiter}>
      <View style={{ alignItems: "center", padding: 2, margin: 2, }}>
        <Image
          style={styles.arbiterImage}
          source={{uri: ARBITER_PICTURE_URL}}
        />
      </View>
      <View style={{ backgroundColor: "slategrey", padding: 1 }}>
        <Text>{ARBITER_NAME}</Text>
      </View>
    </View>
  );
} 

