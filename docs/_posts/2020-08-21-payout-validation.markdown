---
layout: post
title:  "Payout Validation"
date:   2020-08-21 11:36:18 -0400
categories: payout validation
---

The funds are released from escrow via cooperation between the players or by submission of a Payout to the arbiter for approval. Here we present all the validations rules applied to Payouts. 

This does not cover the satisfaction of payout scripts, since that depends on the script at hand. This validation is only with respect to the data structure member values, not whether or not the arbiter will approve the payout.

```
// payout data structure
contract: corresponding contract
payout_tx: transaction for which the arbiter's signature is requested
script_signature: input for the contract payout script
```

* `contract` must be the entire corresponding certified contract, including the contract signatures.

* `payout_tx` must be an unsigned bitcoin transaction which only spends from the escrow address output in the contract funding transaction to one or both of the players' addresses. 

* `script_signature` is data provided as input to the contract payout script. it must be less than XXX in length and begin with YYY and have some metadata up front with separator ZZZ. the arbiter will only sign `payout_tx` if `script_signature` satisfies the contract payout script.
