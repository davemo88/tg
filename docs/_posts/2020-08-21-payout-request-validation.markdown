---
layout: post
title:  "Payout Request Validation"
date:   2020-08-21 11:36:18 -0400
categories: payout validation
---

Payout is accomplished via cooperation between the players or by submission of a request to the arbiter. The arbiter only accepts a valid Payout Request. Here we present all the validations rules applied to Payout Requests. 

This does not cover the satisfaction of payout scripts, since that depends on the script at hand. This validation is only with respect to the data structure, not whether or not the arbiter will approve the request.

```
// payout request data structure
contract: corresponding contract
payout_tx: the transaction which will pay out the escrow account 
    * must spend from the funding transaction 
    * may only spend to addresses controlled by the players
script_signature: input for the payout script, most likely a digital signature
    * unused in case of payout by players
```

* `contract` must be the entire corresponding certified contract, including the contract signatures.

* `payout_tx` must be a potentially valid bitcoin transaction which only spends from the escrow address output in the contract `funding_tx` to one or both of the players' addresses.

* `script_signature` is data provided as input to the payout script. it must be less than XXX in length and begin with YYY and have some metadata up front with separator ZZZ
