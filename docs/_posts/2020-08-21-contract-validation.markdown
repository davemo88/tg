---
layout: post
title:  "Contract Validation"
date:   2020-08-21 11:36:18 -0400
categories: contract validation
---

Contract creation is a detailed processes requiring many checks to ensure security and validity. Here we present all the validation rules applied to contracts. This only applies to its data members, not for processes involving contracts, e.g. [signing](http://localhost:4000/validation/2020/08/22/contract-signing.html) or [payout](http://localhost:4000/payout/validation/2020/08/21/payout-request-validation.html).
```
// contract data stucture
p1_id: player one's Player ID
p2_id: player two's Player ID
arbiter_id: arbiter's Arbiter ID
amount: amount to be held in escrow
payout_script: script used by the arbiter to validate payout requests
funding_tx: the transaction to fund the escrow account
contract_sigs: signatures from players and arbiter
```

* `p1_id` and `p2_id` must be `bech32` encoded Player IDs. Player keys used in `funding_tx` must be children of their underlying keys, e.g. keys used for the multisig.

* `arbiter_id` must be a `bech32` encoded Arbiter ID. Arbiter keys used in `funding_tx` must be a child of its underlying key, e.g. for the multisig and the service fee address.

* `amount` cannot be 0 ??? maybe this enables weird shit

* `payout_script` must be a valid TgScript and that is all. However, the arbiter need not certify every valid script and in fact rejects everything that doesn't follow a standard script template.

* `funding_tx` must spend `amount` into a 2-of-3 multisig of which both players and the arbiter are keyholders. It must also spend X% of `amount` into an address controlled by the arbiter as a service fee.

* `contracts_sigs` is initially blank. The process to add signatures is described  in [Contract Signing]({% post_url 2020-08-22-contract-signing %}).
