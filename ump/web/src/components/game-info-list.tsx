import React from 'react';
import Box from '@material-ui/core/Box';
import Typography from '@material-ui/core/Typography';
import { GameInfo } from '../datatypes';
import { GameInfoDisplay } from '../components/game-info-display';

type GameInfoListProps = {
    pubkey: string,
    infos: GameInfo[],
}

export const GameInfoList = (props: GameInfoListProps) => {
    return (
        <Box display="flex" flexDirection="column" alignItems="center">
            <Typography align='center' variant='h4'>
                Games
            </Typography>
            {props.infos.map((info, index) => {
                return <GameInfoDisplay pubkey={props.pubkey} info={info} />
            })}
        </Box>
    )
}
