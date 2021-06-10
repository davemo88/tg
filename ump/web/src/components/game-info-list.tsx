import React from 'react';
import { GameInfo } from '../datatypes';
import { GameInfoDisplay } from '../components/game-info-display';

type GameInfoListProps = {
    infos: GameInfo[],
}

export const GameInfoList = (props: GameInfoListProps) => {
    return (
        <div>
            <b>Game Info List </b>
            {props.infos.map((info, index) => {
                return <GameInfoDisplay info={info} />
            })}
        </div>
    )
}
