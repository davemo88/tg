---
layout: post
title:  "Updated Project Structure"
date:   2020-12-06 11:28:00 -0400
categories: mock
---

The BDK wallet library was refactored heavily / replaced by another library at some point after the last post. Since then, `tg` is refactored to use the new BDK wallet as well to run in a less mocked-up environment using `docker-compose`. The implementation has been broken up into several libraries and executables shown below.

# Libraries
* `tglib`: core library used throughout
* `player-wallet`: library used by player wallet implementations

# Executables
* `player-cli`: cli player wallet
* `player-app`: react native player wallet
* `rbtr-public`: public arbiter service which handles requests from players
* `rbtr-private`: private arbiter service which signs contracts and payouts
* `nmc-id`: mocked-up player identity service
* `referee-signer`: mocked-up signing tool for referees

# Third Party
* `bitcoind`: underlying blockchain. other blockchains may be substituted.
* `electrum`: blockchain querying
* `redis`: transient storage

# Arbiter Service
The current Arbiter service uses:
* `rbtr-public`
* `rbtr-private`
* `nmc-id`
* `bitcoind`
* `electrum`
* `redis`  

See `docker-compose.yml`.

Players then use the service via wallets like `player-cli` or `player-app`. `referee-signer` is used on an ad hoc basis for the moment.

# In the Future

* `namecoind`: can use namecoin as the underlying storage for the player identity service
* `rbtr-address`: address generation service, as outlined in [a previous post]({% post_url 2020-08-28-wallet-details %})
* more referee tools. this is pretty much untouched at the moment

