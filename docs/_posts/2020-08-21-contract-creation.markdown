---
layout: post
title:  "Contract Creation"
date:   2020-08-21 11:36:18 -0400
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

* `p1_id` and `p2_id` must be a `bech32` encoded Player IDs. The player addresses used in the funding tx must be children of their underlying keys.

* `arbiter_id` must be a `bech32` encoded Arbiter ID. The arbiter fee address in the funding_tx must be a child of its underlying key.

* `amount` cannot be 0

* `payout_script` must be a valid TgScript and that is all. However, the arbiter need not certify every valid script and, quite oppositely, requires a standard script template be followed.

* `funding_tx` must spend `amount` into a 2-of-3 multisig of which both players and the arbiter are keyholders. It must also spend X% of `amount` into an address controlled by the arbiter as a service fee.

* `contracts_sigs` is initially blank. The process to add signatures is described  in [Contract Signing]({% post_url 2020-08-22-contract-signing %}).
