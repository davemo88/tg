import React from 'react';
import { GameInfo } from '../datatypes';

type WinnerDisplayProps = {
    info: GameInfo,
}

const sigStyle = {
    fontSize: 'xx-small',
}

export const WinnerDisplay = (props: WinnerDisplayProps) => {
    if (props.info.winner !== null) {
        return (
            <div>
                <div>
                    Winner    
                </div>
                <div>
                    <b>{get_winner_name(props.info)}</b> 
                </div>
                <div style={sigStyle}>
                    <b>sig: {props.info.sig}</b> 
                </div>
            </div>

        )
    } else {
        return <div>Winner TBD</div>
    }
}

const get_winner_name = (info: GameInfo) => {
    if (info.winner == "home") {
        return info.home
    } else if (info.winner == "away") {
        return info.away
    }
}
