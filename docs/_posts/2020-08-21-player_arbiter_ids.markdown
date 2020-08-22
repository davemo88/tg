---
layout: post
title:  "Player and Arbiter IDs"
date:   2020-08-21 12:02:18 -0400
categories: ids
---

We use the [BIP32](https://en.bitcoin.it/wiki/BIP_0032) HD wallet.

There will be at least five keychains from the root:

one used to compute the player id

one to create bitcoin addreses 
one for bitcoin change

one to create namecoin addresses
one for namecoin change

can use the Master Pubkey to allow others to create new addresses, e.g. during contract creation

this way it should be possible to always use fresh addresses but also verify ownership of addresses via checking that the address and the player ID have a common ancestor key i.e. the root key
