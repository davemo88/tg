// toothless rake tournament grounds
// namecoin powered online tournament system
// players register namecoin names
// then use multisigs anywhere referring to namecoin names
// as escrows for tournament prize pools
// final results written to namecoin chain (what were you thinking?)
//

use std::collections::HashMap;

const BYTE_ARRAY_SIZE: usize = 64;

type ChallengeId = u64;

type Pubkey = [u8; BYTE_ARRAY_SIZE];
type MatchAddress = [u8; BYTE_ARRAY_SIZE];
type Transaction = [u8; BYTE_ARRAY_SIZE];
type MatchData = [u8; BYTE_ARRAY_SIZE];
type MatchHash = [u8; BYTE_ARRAY_SIZE];
type Signature = [u8; BYTE_ARRAY_SIZE];
type PayoutScript = [u8; BYTE_ARRAY_SIZE];
type Bux = u32;

// maybe a namecoin address for the pubkey for a free reputation system
struct Player {
    pubkey: Pubkey,
}

impl KeyHolder for Player {}

struct Referee {
    pubkey: Pubkey,
}

impl KeyHolder for Referee {}

trait KeyHolder {
    fn sign(data_to_sign: [u8; BYTE_ARRAY_SIZE]) -> Signature {
        [1; BYTE_ARRAY_SIZE]
    }

    fn verify(signature: Signature, pubkey: Pubkey) -> bool {
// TODO: really do it
        let sum: u8 = signature.iter().sum();
        if sum == 0 {
            return false
        }
        else {
            return true
        }
    }
}

struct Challenge {
    id: ChallengeId,
    escrow: MultisigEscrow,
    funding_tx: Transaction,
    payout_script: PayoutScript, 
}

impl Challenge {
    pub fn is_signed() -> bool {
        false
    }
}

struct Resolution {
    challenge_id: ChallengeId,
    payout_tx: Transaction,
}

struct MultisigEscrow {
    pubkey: Pubkey, 
    players: Vec<Player>,
    referees: Vec<Referee>,
}

impl MultisigEscrow {
    pub fn new(players: Vec<Player>, referees: Vec<Referee>) -> Self {
        MultisigEscrow {
            pubkey: MultisigEscrow::make_pubkey(&players, &referees),
            players: players,
            referees: referees,
        }
    }

    fn make_pubkey(players: &Vec<Player>, referees: &Vec<Referee>) -> Pubkey {
        [0; BYTE_ARRAY_SIZE]
    }

    fn balance(&self) -> Bux {
// get escrow balance, likely stored on blockchain
        0
    }
}
