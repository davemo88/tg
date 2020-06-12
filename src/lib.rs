// toothless rake tournament grounds
// namecoin powered online tournament system
// players register namecoin names
// then use multisigs anywhere referring to namecoin names
// as escrows for tournament prize pools
// final results written to namecoin chain (what were you thinking?)
//
//
use secp256k1::{
    Secp256k1,
    Message,
};
use rand::Rng;
use bitcoin::{
    Transaction,
    Address,
    PublicKey,
    Network,
};

pub struct PayoutScript;
pub struct PayoutScriptSigHash;

pub struct Challenge {
    escrow: MultisigEscrow,
    funding_tx: Transaction,
    payout_script_sig_hash: PayoutScriptSigHash,
}

pub struct Referee;

impl Referee { 
    fn create_pubkey(&self) {

    }

    fn approve_payout(&self, payout: &mut Payout) {

    }
}

//NOTE: could use dummy tx requiring signing by referee to embed info in tx
impl Challenge {
    pub fn new(id: u64, escrow: MultisigEscrow, funding_tx: Transaction, payout_script_sig_hash: PayoutScriptSigHash) -> Self {
        Challenge {
            escrow: escrow,
            funding_tx: funding_tx,
            payout_script_sig_hash: payout_script_sig_hash,
        }
    }
}

//NOTE: create all the possible payout txs beforehand and then branch on something for a basic payout
//script, e.g. in 1v1 winner takes all all to A or B based on some value,
//could require signature from the TO. 
//if you need resolution then somebody has to look it up the value
struct Payout {
    payout_tx: Transaction,
    challenge: Challenge,
}

impl Payout {
    pub fn new(payout_tx: Transaction, challenge: Challenge) -> Self {
        Payout {
            payout_tx: payout_tx,
            challenge: challenge,
        }
    }
}

pub struct MultisigEscrow {
    address: Address, 
    players: Vec<PublicKey>,
    referees: Vec<PublicKey>,
}

impl MultisigEscrow {
    pub fn new(address: Address, players: Vec<PublicKey>, referees: Vec<PublicKey>, signatures_required: u8,) -> Self {
        MultisigEscrow {
            address: address,
            players: players,
            referees: referees,
        }
    }

    pub fn is_signed_by_players(&self, transaction: Transaction) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn make_keypair() {

    }

    #[test]
    fn make_multisig() {

    }
}
