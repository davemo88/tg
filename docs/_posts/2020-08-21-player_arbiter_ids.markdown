---
layout: post
title:  "Player and Arbiter IDs"
date:   2020-08-21 12:02:18 -0400
categories: ids
---

The Player ID is a [bech32](https://en.bitcoin.it/wiki/BIP_0173) encoded string of a hash of the player's Master Pubkey. The human readable portion indicates that the string contains Crypto Escrow player data and the data portion contains the pubkey hash and perhaps some other data, just like a bitcoin address. The underlying key must be compatible with bitcoin so that it can be used to find a corresponding name registered in the namecoin blockchain, the Player Name.

The Arbiter ID is similar, but the human readable part indicates that it contains arbiter data instead. This could also be done by flipping a bit in the data portion but changing the human readable part will reduce user errors and ease validation.

We use the [BIP32](https://en.bitcoin.it/wiki/BIP_0032) HD wallet for its implementation of hierarchical deterministic wallets.

There will be at least four keychains from the root:

one to create bitcoin addreses 
one for bitcoin change

one to create namecoin addresses
one for namecoin change

can use the Master Pubkey to allow others to create new addresses, e.g. during contract creation. this is great because it allows us to make contract funding tx's use unique for each party

this way it should be possible to always use fresh addresses but also verify ownership of addresses via checking that the address and the player ID have a common ancestor key i.e. the root key
