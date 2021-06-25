import React, { useState } from 'react';
import Typography from '@material-ui/core/Typography';
import Card from '@material-ui/core/Card';
import Paper from '@material-ui/core/Paper';
import Box from '@material-ui/core/Box';
import Button from '@material-ui/core/Button';
import Collapse from '@material-ui/core/Collapse';
import { styled } from '@material-ui/core/styles';
import { TeamAvatar } from '../components/team-avatar';
import { GameInfo, Winner, RelativeLoc, toEvent } from '../datatypes';

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
    minWidth: 120,
})

const GameInfoCard = styled(Card)({
    backgroundColor: "LightGrey",
    margin: 10,
    padding: 3,
    width: "90%",
    display: "flex",
    alignItems: "center",
    flexDirection: "column",
    maxWidth: 420,
});

const DetailsPaper = styled(Paper)({
    backgroundColor: 'DarkGrey',
    padding: 5,
    margin: 5,
    maxWidth: 400,
})

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
                    <Typography align='center' variant='body2'>Home</Typography>
                    <TeamAvatar team={props.info.home} />
                </TeamBox>
                <VsBox>
                    <VsTypo>VS</VsTypo>
                </VsBox>
                <TeamBox style={{backgroundColor: gameTeamColor("away", props.info.winner)}}>
                    <Typography align='center' variant='body2'>Away</Typography>
                    <TeamAvatar team={props.info.away} />
                </TeamBox>
            </Box>
            <Typography align='center' variant='body2'>{props.info.date}</Typography>
            <Button 
                onClick={() => setCollapsed(!collapsed)}
            >
        { collapsed ? "Show" : "Hide" } Event Details
            </Button> 
            <Collapse in={!collapsed}>
                <DetailsPaper>
                    <pre style={{ whiteSpace: "pre-wrap", wordWrap: "break-word"}}>{JSON.stringify(toEvent(props.info), null, 2)}</pre>
                </DetailsPaper>
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
