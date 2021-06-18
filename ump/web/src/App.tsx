import React, { useEffect, useState } from 'react';
//import logo from './logo.svg';
//import './App.css';
import CssBaseline from '@material-ui/core/CssBaseline';
import Container from '@material-ui/core/Container';
import { GameInfo } from './datatypes';
import { GameInfoList } from './components/game-info-list';

function App() {
    const [error, setError] = useState<Error|null>(null);
    const [isLoaded, setIsLoaded] = useState(false);
    const [infos, setInfos] = useState<GameInfo[]>([]);

    useEffect(() => {
        fetch("http://0.0.0.0:6969/game-info")
            .then(res => res.json())
            .then((result) => {
                setIsLoaded(true);
                console.log("result", result);
                setInfos(result.data.map((info: any) => {
                    console.log(info.outcome_tokens["HomeWins"]);
                    let home_sig = info.outcome_tokens["HomeWins"][2];
                    let away_sig = info.outcome_tokens["AwayWins"][2];
                    let winner_sig: [string, string] | null = null;
                    if (home_sig) {
                        winner_sig = ["home", home_sig];
                    } else if (away_sig) {
                        winner_sig = ["away", away_sig];
                    }
                    return {
                        date: info.date,
                        home: info.home,
                        away: info.away,
                        home_token: info.outcome_tokens["HomeWins"][1],
                        away_token: info.outcome_tokens["AwayWins"][1],
                        winner: winner_sig ? winner_sig[0]: null,
                        sig: winner_sig ? winner_sig[1]: null,
                    };
                    
                }).sort((a: GameInfo,b: GameInfo) => (a.date < b.date)));
            })
            .catch((e) => {
                console.error(Error(e));
                setError(e);
            });
    }, []);

    if (error) {
        return <div>Error: {error!.message}</div>
    }
    else if (!isLoaded) {
        return <div>Loading...</div>
    } else {
        return (
            <React.Fragment>
                <CssBaseline />
                <div className="App">
                    <Container>
                        <header className="App-header">
                            <GameInfoList infos={infos} />
                        </header>
                    </Container>
                </div>
            </React.Fragment>
        );
    }

}

export default App;
