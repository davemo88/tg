import React from 'react';
import { Button, Text, View, } from 'react-native';

import { styles } from '../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, } from '../redux';
import { Player, Contract, ContractStatus } from '../datatypes';

import { PlayerPortrait } from './player-portrait';

export interface PlayerSelectorProps {
  playerIds: string[];
  selectedPlayerId: string;
  setSelectedPlayerId: (newPlayerId: string) => void;
}

// TODO: fix moving buttons based on name length
export const PlayerSelector: React.FC<PlayerSelectorProps> = (props) => {
  const selectedPlayer = playerSelectors.selectById(store.getState(), props.selectedPlayerId);

  return (
    <View style={styles.playerSelector}>
      <PlayerSelectorButton forward={false} selectedPlayerId={props.selectedPlayerId} setSelectedPlayerId={props.setSelectedPlayerId} playerIds={props.playerIds} />
      <PlayerPortrait 
        name={selectedPlayer.name}
        pictureUrl={selectedPlayer.pictureUrl}
      />
      <PlayerSelectorButton forward={true} selectedPlayerId={props.selectedPlayerId} setSelectedPlayerId={props.setSelectedPlayerId} playerIds={props.playerIds} />
    </View>
  );
}

interface PlayerSelectorButtonProps {
  forward: boolean;
  playerIds: string[];
  selectedPlayerId: string;
  setSelectedPlayerId: (newPlayerId: string) => void;
}

const PlayerSelectorButton: React.FC<PlayerSelectorButtonProps> = (props) => {
  const playerIndex = props.playerIds.findIndex((playerId) => playerId === props.selectedPlayerId );

  return (
    <View style={{ justifyContent: 'center', padding: 10 }}>
      <Button
        title={ props.forward ? ">" : "<" } 
        onPress={() => {
          let newPlayerIndex = props.forward ? playerIndex+1 : playerIndex-1;
          newPlayerIndex = (newPlayerIndex + props.playerIds.length) % props.playerIds.length;
          props.setSelectedPlayerId(props.playerIds[newPlayerIndex]);
        }}
      />
    </View>
  );
}

