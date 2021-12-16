# Crypto Escrow 

An arbitration service for cryptocurrency escrow, especially for transactions related to video games.

# Architecture

There are 3 main entities: players, oracles, and the arbiter. Players create contracts based on oracle events and submit them to the arbiter for approval. Each contract is backed by a 2-of-3 multisig with keys from 2 players and the arbiter. A player can later submit an approved contract to the arbiter for resolution if the other player is unavailable or uncooperative. The submission also includes some data from the oracle that tells the arbiter how the contract should be resolved.

The crux of each contract is the Payout Script. It is written in `TgScript`, a Bitcoin Script-inspired mini language in which player specify the terms of the contract. For now, the arbiter will only approve contracts which use a standard payout script. The standard script is a simple winner-takes-all and the arbiter will only release funds from the escrow if the requested payout transaction matches a specified one and is accompanied by a corresponding signature from the oracle.

The wallet allows players to register names on the namecoin chain to simplify exchange of information. Players post the necessary information to create contracts under their registered names, allowing other players to find their info and propose contracts. The contracts are based on events announced/attested to by oracles.

## modules

### tglib
Common data structures and utilities used in other modules.
### player-wallet
This is a wallet library that handles the contract lifecycle i.e. creation, signing, and verification, communication with the arbiter, and registering player names.
### player-cli
This is a cli wrapper around `player-wallet`.
### rbtr-public
This is the public facing arbiter service that accepts requests from players.
### rbtr-private
This is the arbiter's signing service. It is isolated because it requires private keys.
### nmc-id
Name registration service. Since players need to exchange some data to create a contract, this lets them do it under human readable names instead of public keys or addresses.
### exchange
This little service handles the data exchange between players during contract setup. It uses `nmc-id` for authentication.
### ump
This is an example oracle service for Baseball.
#### ump-publisher
This service publishes the latest Baseball results.
#### ump-signer
This services signs the outcomes of resolved events. Since it requires private keys, it runs in isolation similarly to `rbtr-private`.
#### ump-web
Web frontend which reads data from `ump-publisher`.
# to run:

Start the arbitration service (the arbiter) with `docker-compose up` from within the main project directory. Start the demo oracle from the `ump` directory similarly. It attests to baseball game outcomes. Unforunately it doesn't do much outside baseball season :)
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
