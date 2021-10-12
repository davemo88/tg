# Crypto Escrow 

An arbitration service for cryptocurrency escrow, especially for transactions related to video games.

This wallet allows players to register names on the namecoin change to simplify exchange of information. Players post the necessary information to create contracts under their registered names, allowing other players to find their info and propose contracts. The contracts are based on events announced/attested to by oracles. 

# to run:

Start the arbitration service (the arbiter) with `docker-compose up` from within the main project directory. Start the demo oracle from the `ump` directory similarly. It attests to baseball game outcomes.
## cli
Run `./cli.sh` to open a repl for cli wallet. To get started, run `init`, `fund`, and then `balance` to create a funded wallet. You will need to create a password. You can work with multiple wallets with the `wallet-dir` option.
Register a name with `player register` and then post your contract info with `player post`.
Assuming another player, say Bob, has posted contract info, you can create a new contract with `contract new Bob ...`. You can get an oracle event from `http://localhost:3000`. Paste the event json into the terminal as part of the `contract new` command.

Players need to exchange the signed contract with `contract send` and `contract receive` and sign it with `contract sign`.

Once both players have signed the contract, they submit it to arbiter with `contract submit`.

After obtaining the arbiter's signature, they broadcast the funding transaction with `contract broadcast`. Once the event is resolved, they can create payouts with `payout new`. They can payout cooperatively by both signing the payout or they can retrieve a token from the oracle and submit the payout to the arbiter with `payout submit`. Finally the payout transaction is broadcast with `payout broadcast`.

## android
To run the mobile app you will need Android Studio with the NDK installed. Switch to the `PlayerApp` directory and run 
```
$ npm i
$ npm start
```
Open a new terminal and run `npm run-script android`.
