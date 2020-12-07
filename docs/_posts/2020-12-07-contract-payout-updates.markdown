---
layout: post
title:  "Updated Contracts and Payouts"
date:   2020-12-07 11:28:00 -0400
categories: contract payout
---

In the current implementation, contracts and payouts use [Partially Signed Transactions](https://github.com/bitcoin/bips/blob/master/bip-0174.mediawiki) in raw formats. This is quite convenient because we need to pass around transactions for signing by multiple parties in several cases. However this also makes transaction handling slightly more complicated in clients that must create contracts and payouts.

```
Updated Contract Struct:
{
    p1_pubkey: player 1's pubkey
    p2_pubkey: player 2's pubkey
    arbiter_pubkey: arbiter's pubkey
    funding_tx: partially signed transaction funding the contract
    contract_sigs: list of signatures in order p1, p2, arbiter
}
```
This data makes it easier to validate the contract. Use of PlayerIDs in the contract body prevent pubkey-based validation since they are pubkey hashes. The signature list seemed easier at the time but the inclusion of the pubkeys makes nested signatures possible to verify with the contract alone.

The partially signed transaction isn't strictly necessary here, since the arbiter does not require the funding transaction to be signed upon contract submission. However it simplifies signing the funding transaction since it can be done as the contract is signed.

```
Updated Payout Struct:
{
    contract: associated contract
    payout_tx: partially signed transaction paying out the contract
    script_sig: payout script signature
}
```
The partially signed transaction is necessary here since the player submitting the payout must sign the payout transaction before the arbiter. The escrow address is formed using the pubkeys in order p1, p2, arbiter and spends from a multisig must be signed in accordance with the order used in formation.
