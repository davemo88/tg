---
layout: post
title:  "Contract Signing"
date:   2020-08-22 11:36:18 -0400
categories: validation
---

A contract requires all three parties' signatures to be certified, and the arbiter will only sign payouts for certified contracts. The parties sign the contract in a specific order: Player 1, Player 2, and finally the Arbiter. 

Player 1 computes the contract hash and signs it. Player 2 verifies Player 1's signature, then signs on top. Finally the Arbiter verifies both players' signatures and signs on top before returning the now-certified contract to the players. At this point the players are free to broadcast the funding transaction, play their game, and request payout. The arbiter will process payout reqeusts for the contract.

When a signature is added to the contract, it is added on top of the previous signature. "on top" means using the previous signature as the message to sign. 

Player 1 does not have to verify any signatures because they are the first to sign. They simply sign the contract hash.

Player 2 must verify Player 1's signature. They compute the contract hash and then verify the given contract signature given Player 1's pubkey. Signatures are not part of the contract hash so signing the contract does not modify its hash.

The arbiter must verify both players' signatures. It computes the contract hash, decrypts the given signature using Player 2's pubkey, and then verifies that it is Player 1's signature on the contract hash. If so, this implies that Player 2's signature is valid as well.

Each party also validates the rest of the contract before signing, ensuring it follows all the rules detailed in [Contract Creation](http://localhost:4000/validation/2020/08/21/contract-creation.html).
