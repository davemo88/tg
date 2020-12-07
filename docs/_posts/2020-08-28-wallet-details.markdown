---
layout: post
title:  "Wallet Implementation Details"
date:   2020-08-28 11:28:00 -0400
categories: wallet
---

Crypto Escrow relies on wallet applications to implement its protocol.

## Arbiter Wallets 

The arbiter uses several wallet apps. One such app provides addresses for use in contracts. Another validates and signs contracts. This way the cryptographic privileges of the apps can be individually defined and restricted accordingly. Keys are insulated from network access via the application structure but also by intermediary services which pass data from an internet-facing application to the insulated one. These services could be simple proxies or work queues or contain application logic, preferably the former.

### Address Provider 
The Address Provider uses an extended public key to create addresses for use in contracts. A level of insulation between the extended public key and the internet (and therefore the players) is desirable because if the extended public key is compromised, a single child private key can be used to deduce the corresponding extended private key and therefore the keys of all dispensed addresses.

Instead of accessing the provider directly, an Address Dispenser service dispenses addresses to players upon request. The dispenser gets its addresses from a queue monitored and periodically restocked by the provider. 

`
Address Provider <-> Address Queue <-> Address Dispenser <-> Players
`

### Contract Signer
The Contract Signer requires private key access to sign contracts and must be kept maximally secure. Like the Address Provider, the Contract Signer is not directly accessible over the network. Players submit their contracts to a public service, which enters them in a queue monitored by the signer. 

`
Contract Signer <-> Contract Queue <-> Contract Handling <-> Players
`

The signer removes contracts in the queue for validation and, potentially, signing. Signed contracts are placed into a separate queue to be sent back to the players.

`
Contract Signer <-> Signed Contract Queue <-> Contract Handling <-> Players
`

## Player Wallets
In its most basic form, the player wallet is an extended normal crypto wallet that facilitates the creation and management of contracts, payouts, and the related transactions in the underlying cryptocurrency. Beyond this the wallet can be tailored to a specific use case, e.g. because each contract follows the same pattern in that case.
