import React from 'react';
import { Button, Text, View, } from 'react-native';

import { styles } from '../styles';

import { useDispatch } from 'react-redux';
import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, removePlayer } from '../redux';
import { Player, Contract } from '../datatypes';

import { PlayerPortrait } from './player-portrait';

export interface PlayerSelectorProps {
    playerNames: string[];
    selectedPlayerName: string;
    setSelectedPlayerName: (newPlayerName: string) => void;
    allowRemoval: boolean;
}

// TODO: fix moving buttons based on name length
export const PlayerSelector: React.FC<PlayerSelectorProps> = (props) => {
    const dispatch = useDispatch();
    const selectedPlayer = playerSelectors.selectById(store.getState(), props.selectedPlayerName);
    const [removingPlayer, setRemovingPlayer] = React.useState(false);

    return (
      <View style={styles.playerSelector}>
        <PlayerSelectorButton forward={false} selectedPlayerName={props.selectedPlayerName} setSelectedPlayerName={props.setSelectedPlayerName} playerNames={props.playerNames} />
        <PlayerPortrait 
          name={selectedPlayer.name}
          pictureUrl={selectedPlayer.pictureUrl}
        />
        <PlayerSelectorButton forward={true} selectedPlayerName={props.selectedPlayerName} setSelectedPlayerName={props.setSelectedPlayerName} playerNames={props.playerNames} />
      </View>
    );
}

interface PlayerSelectorButtonProps {
  forward: boolean;
  playerNames: string[];
  selectedPlayerName: string;
  setSelectedPlayerName: (newPlayerName: string) => void;
}

const PlayerSelectorButton: React.FC<PlayerSelectorButtonProps> = (props) => {
  const playerIndex = props.playerNames.findIndex((playerName) => playerName === props.selectedPlayerName );

  return (
    <View style={{ justifyContent: 'center', padding: 10 }}>
      <Button
        title={ props.forward ? ">" : "<" } 
        onPress={() => {
          let newPlayerIndex = props.forward ? playerIndex+1 : playerIndex-1;
          newPlayerIndex = (newPlayerIndex + props.playerNames.length) % props.playerNames.length;
          props.setSelectedPlayerName(props.playerNames[newPlayerIndex]);
        }}
      />
    </View>
  );
}

