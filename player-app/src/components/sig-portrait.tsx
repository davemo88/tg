import React from 'react';
import { Text, Image, View, } from 'react-native';

import { styles } from '../styles.ts';

export interface SigPortraitProps {
  name: string;
  pictureUrl: string;
  isSigned: bool;
}

export const SigPortrait: React.FC<SigPortraitProps> = (props) => {
    const imageStyle = props.isSigned ? styles.smallEmote : { width: 28, height: 28, opacity: 0.5, tintColor: 'lightcoral' };
  //  const imageStyle = props.isSigned ? styles.smallEmote : { ...styles.smallEmote, opacity: 0.5, tintColor: 'lightcoral' };

  return (
    <View style={styles.sigPortrait}>
      <View style={styles.smallEmote}>
        <Image
          style={imageStyle}
          source={props.pictureUrl}
        />
      </View>
    </View>
  );
} 

