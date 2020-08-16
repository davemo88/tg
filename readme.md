#Tournament Grounds Toothless Rake 

an arbitration service for cryptocurrency escrow, especially for transactions related to video games

## Concepts
* Escrow Contract - an escrow contract is an agreement about how to use a 2-of-3 multsig account (escrow account) containing keys from two players as well as a third party known as the arbiter. 

as part of formation of the escrow contract, the players draft a transaction to fund (funding_tx) the account as well as a script used by the arbiter to settle disputes between the players over how to pay out the account (payout_script). 

because signatures from only 2 of the 3 keyholders are required to spend from the account, the players may pay out the account at any time if they cooperate. in case of a dispute, the payout script allows the players to obtain a signature from the arbiter if the conditions of the script can be satisfied. certain standard script templates are defined but players can create their own.

An Escrow Contract requires signatures from both players as well as the arbiter.

Escrow Contract data stucture:
p1_pkh: player one's pubkey hash 
p2_pkh: player two's pubkey hash
arbiter_pkh: arbiter's pubkey hash
funding_tx: the transaction which will fund the escrow account 
payout_script: the script used by the arbiter to determine if payout requests are valid
signatures: signed hash of the other attributes. this hash is called the contract id or cxid
//TODO: look up how txid is computed for bitcoin transactions

p1 signs both the funding_tx as well as the contract (cxid). p2 verifies these signatures, then signs thefunding_tx and puts his signature the contract on top of p1's. at this point the funding tx is valid because both players have signed. however the funding tx cannot already be mined for a contract to be approved by the arbiter. the arbiter confirms that the funding tx is valid (i.e. signed by both players but not in the blockchain) and that both player signatures are on the contract before signing on top.

now the players can broadcast the funding tx. to pay out the contract, the players may simply sign a payout request together or use the arbiter service to resolve a dispute.

to use the arbiter, one player prepares a payout tx and a script signature as well as the original contract including all the contract signatures. the arbiter uses the contract signatures to verify that the contract is certified by the arbiter and then uses the contract's script to determine the validity of the payout request, passing the script signature as input and using the payout tx as context.

in the standard case, the contract script will require the payout tx to spend from the funding tx in a specific way given the value of the script sig


```

```

* Payout Request - a request to spend money from the escrow account. The players may pay out the account themselves by cooperating. Otherwise they send a Payout Request to the arbiter. 
* Script - an bitcoin-script inspired scripting language. instead of validating bitcoin transaction, the script is used to validate Payout Requests.

## Primary Use Case Summary

1. player creates escrow and sends it to other player
2. other player accepts or declines it. if they accept, send contact to arbiter
3. arbiter certifies the contract and sends it back to players
4. players broadcast the funding tx
5. players play their match
6. players payout the contract unless there is a disputre.
7. if there is a dispute, one player sends a payout request to the arbiter to payout the contract
8. contract resolved after payout

### Contract Details
#### Creation
### Payout Details

## Potential Improvements

### styled components 
https://styled-components.com/
suggested by Geoff to get rid of my inline styling

