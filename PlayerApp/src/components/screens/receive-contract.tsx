import React from 'react';
import { View } from 'react-native';

import { styles } from '../../styles';
import { CheckMail } from '../check-mail';

export const ReceiveContract = () => {
    return (
        <View style={styles.container}>
            <CheckMail then={()=>{}} />
        </View>
    );
}
