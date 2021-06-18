import React, { useState } from 'react';
import Typography from '@material-ui/core/Typography';
import Card from '@material-ui/core/Card';
import Box from '@material-ui/core/Box';
import Button from '@material-ui/core/Button';
import Collapse from '@material-ui/core/Collapse';
import { styled } from '@material-ui/core/styles';
import { TeamAvatar } from '../components/team-avatar';
import { GameInfo, Winner, RelativeLoc } from '../datatypes';

type GameInfoDisplayProps = {
    info: GameInfo,
}

const VsBox = styled(Box)({
    display: 'inline-block',
    margin: '10px',
    fontSize: 'small',
})

const VsTypo = styled(Typography)({
    align: 'center',
    variant: 'body2',
})

const TeamBox = styled(Box)({
    display: 'inline-block',
    margin: '10px',
})

const GameInfoCard = styled(Card)({
     backgroundColor: "LightGrey",
     margin: 10,
     width: "90%",
     display: "flex",
     alignItems: "center",
     flexDirection: "column",
     maxWidth: 420,
});

const gameTeamColor = (relLoc: RelativeLoc, winner: Winner) => {
    if (winner === null) {
        return 'default'
    } else {
        return relLoc === winner ? 'green' : 'red'
    }
}

export const GameInfoDisplay = (props: GameInfoDisplayProps) => {
    const [collapsed, setCollapsed] = useState(true);
    return (
        <GameInfoCard>
            <Box>
                <TeamBox style={{backgroundColor: gameTeamColor("home", props.info.winner)}}>
                    <TeamAvatar team={props.info.home} />
                </TeamBox>
                <VsBox>
                    <VsTypo>VS</VsTypo>
                </VsBox>
                <TeamBox style={{backgroundColor: gameTeamColor("away", props.info.winner)}}>
                    <TeamAvatar team={props.info.away} />
                </TeamBox>
            </Box>
            <Typography align='center' variant='body2'>{props.info.date}</Typography>
            <Button 
                onClick={() => setCollapsed(!collapsed)}
            >
                { props.info.winner ? "Show Signature" : "Show Tokens" }
            </Button> 
            <Collapse in={!collapsed}>
                <GameInfoDetails info={props.info} />
            </Collapse>
        </GameInfoCard>
    )
}

const getWinnerName = (info: GameInfo) => {
    switch (info.winner) {
        case "home":
            return info.away.name
        case "away":
            return info.home.name
        default:
            return null
    }
}

const GameInfoDetails = (props: GameInfoDisplayProps) => {
    if (props.info.winner) {
        return( 
            <Box style={{ maxWidth: 400 }}>
                <Typography>{getWinnerName(props.info)} won:</Typography>
                <Typography style={{ wordWrap: "break-word" }}>{props.info.sig}</Typography>
            </Box>
        )            
    } else {
        return(
            <Box style={{ maxWidth: 400 }}>
                <Typography>{props.info.home.name} win:</Typography>
                <Typography style={{ wordWrap: "break-word" }}>{props.info.home_token}</Typography>
                <Typography>{props.info.away.name} win:</Typography>
                <Typography style={{ wordWrap: "break-word" }}>{props.info.away_token}</Typography>
            </Box>
        )
    }
}
