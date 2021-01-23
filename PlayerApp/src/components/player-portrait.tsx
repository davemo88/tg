import React from 'react';
import { Text, Image, View, } from 'react-native';

import { styles } from '../styles.ts';

export interface PlayerPortraitProps {
  name: string;
  pictureUrl: string;
}

export const PlayerPortrait: React.FC<PlayerPortraitProps> = (props) => {
  return (
    <View style={styles.player}>
      <View style={{ alignItems: "center" }}>
        <Image
          style={styles.mediumEmote}
          source={{uri: props.pictureUrl}}
        />
      </View>
      <View style={{ alignItems: 'center', backgroundColor: "slategrey", padding: 1 }}>
        <Text style={{ fontSize: 17 }}>{props.name}</Text>
      </View>
    </View>
  );
} 


