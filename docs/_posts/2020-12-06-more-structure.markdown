---
layout: post
title:  "Moar Structure"
date:   2020-12-06 11:28:00 -0400
categories: mock
---

The BDK wallet library was refactored heavily / replaced by another library at some point after the last post. Since then, `tg` is refactored to use the new BDK wallet which is quite nice and uses descriptors. The overall structure has become better defined.

# Libraries
* `tglib`: core library used throughout
* `player-wallet`: library used by player wallet implementations

# Executables
* `player-cli`: cli player wallet
* `player-app`: react native player wallet
* `rbtr-public`: public arbiter service which receives and handles requests from players
* `rbtr-private`: private arbiter service which signs contracts and payouts
* `nmc-id`: mocked-up player identity service
* `referee-signer`: mocked-up signing tool for referees

# Third Party
* `bitcoind`: underlying blockchain. other blockchains can be substituted.
* `electrum`: blockchain querying
* `redis`: transient storage

# Arbiter Service
The complete Arbiter service runs:
* `rbtr-public`
* `rbtr-private`
* `nmc-id`
* `bitcoind`
* `electrum`
* `redis`

Players then use the service via wallets like `player-cli` or `player-app`. `referee-signer` is used in an ad hoc basis for the moment.
