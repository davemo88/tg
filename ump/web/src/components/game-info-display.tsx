import React from 'react';
import { GameInfo, Winner } from '../datatypes';
import Typography from '@material-ui/core/Typography';
import { TeamAvatar } from '../components/team-avatar';
import { WinnerDisplay } from './winner-display';

type GameInfoDisplayProps = {
    info: GameInfo,
}

const displayStyle = {
    display: 'inline-block',
    margin: '10px',
}

const vsStyle = {
    display: 'inline-block',
    margin: '10px',
    fontSize: 'small',
}

export const GameInfoDisplay = (props: GameInfoDisplayProps) => {
    return (
        <div style={{
            backgroundColor: "LightGrey",
            margin: 10,
        }}>
            <div>
                <Typography align='center' variant='h6'>{props.info.date}</Typography>
            </div>
            <div style={displayStyle}>
                <TeamAvatar team={props.info.home} />
            </div>
            <div style={vsStyle}>
                <div>
                    VS
                </div>
            </div>
            <div style={displayStyle}>
                <TeamAvatar team={props.info.away} />
            </div>
        </div>
    )
}

type TokenDisplayProps = {
    token: string,
}

const tokenStyle = {
    fontSize: 'x-small',
}

const TokenDisplay = (props: TokenDisplayProps) => {
    return <div style={tokenStyle}>token: {props.token}</div> 
}
