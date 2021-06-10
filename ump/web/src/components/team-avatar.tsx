import React from 'react';
import Typography from '@material-ui/core/Typography';
import { Team } from '../datatypes';

type TeamAvatarProps = {
    team: Team,
}

export const TeamAvatar = (props: TeamAvatarProps) => {
    return (
        <div style={{
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          flexDirection: "column"
        }}>
            <img src="https://via.placeholder.com/80" alt="logo" />    
            <Typography variant='body2'>{props.team.location}</Typography>
            <Typography variant='body2'>{props.team.name}</Typography>
        </div>
    )
}
