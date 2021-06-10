import React from 'react';
import Box from '@material-ui/core/Box';
import Typography from '@material-ui/core/Typography';
import { GameInfo } from '../datatypes';
import { GameInfoDisplay } from '../components/game-info-display';

type GameInfoListProps = {
    infos: GameInfo[],
}

export const GameInfoList = (props: GameInfoListProps) => {
    return (
        <div style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
        }}>
            <Typography align='center' variant='h4'>
                <div style={{color: "White"}}>
                    Games
                </div>
            </Typography>
            {props.infos.map((info, index) => {
                return <GameInfoDisplay info={info} />
            })}
        </div>
    )
}
