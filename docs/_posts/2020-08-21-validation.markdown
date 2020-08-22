---
layout: post
title:  "Security and Validation"
date:   2020-08-21 11:36:18 -0400
categories: contract creation validation
---

At each step in the process, the parties perform validation. This includes verifying digital signatures as well as confirming senders, recipients, and amounts of funds.

## Contract Creation
```
// contract data stucture
p1_pkh: player one's pubkey hash 
p2_pkh: player two's pubkey hash
arbiter_pkh: arbiter's pubkey hash
amount: amount to be held in escrow
payout_script: script used by the arbiter to validate payout requests
funding_tx: the transaction to fund the escrow account
contract_sigs: signatures from players and arbiter
```

* `p1_pkh` and `p2_pkh` must be valid Player ID (pubkey hash) sharing underlying keys with the players' addresses in the `funding_tx`
* `arbiter_pkh` must be a valid Arbiter ID (pubkey hash)
