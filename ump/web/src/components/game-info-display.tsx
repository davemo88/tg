import React from 'react';
import { GameInfo, Winner } from '../datatypes';
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
        <div>
            <div style={displayStyle}>
                <div>
                    <b>{props.info.home}</b>
                </div>
                <TokenDisplay token={props.info.home_token} />
            </div>
            <div style={vsStyle}>
                <div>
                    VS.
                </div>
                <div>
                    {props.info.date}
                </div>
            </div>
            <div style={displayStyle}>
                <div>
                <b>{props.info.away}</b></div>
               <TokenDisplay token={props.info.away_token} />
            </div>
            <WinnerDisplay info={props.info} />
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
