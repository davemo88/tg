import React from 'react';
import { Button, Text, View, } from 'react-native';

import { styles } from '../styles.ts';

import { store, playerSlice, playerSelectors, localPlayerSlice, localPlayerSelectors, contractSelectors, contractSlice, selectedLocalPlayerIdSlice, } from '../redux.ts';
import { Player, LocalPlayer, Contract, ContractStatus, getContractStatus } from '../datatypes.ts';

import { PlayerPortrait } from './player-portrait.tsx';

export interface PlayerSelectorProps {
  playerIds: string[];
  selectedPlayerId: string;
  setSelectedPlayerId: (newPlayerId: string) => void;
}

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
  forward: bool;
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

