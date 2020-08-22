---
layout: post
title:  "Contract Signing"
date:   2020-08-22 11:36:18 -0400
categories: validation
---

Contract creation is a detailed processes requiring many checks to ensure security. Here we outline each step in full detail.

```
// contract data stucture
p1_id: player one's Player ID
p2_id: player two's Player ID
arbiter_id: arbiter's pubkey hash
amount: amount to be held in escrow
payout_script: script used by the arbiter to validate payout requests
funding_tx: the transaction to fund the escrow account
contract_sigs: signatures from players and arbiter
```
