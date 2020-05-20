// toothless rake tournament grounds
// namecoin powered online tournament system
// players register namecoin names
// then use multisigs anywhere referring to namecoin names
// as escrows for tournament prize pools
// final results written to namecoin chain

const PUBKEY_SIZE: usize = 64;
const SIGNATURE_SIZE: usize = 64;
const MATCH_HASH_SIZE: usize = 64;
const MATCH_DATA_SIZE: usize = 64;

type Pubkey = [u8; PUBKEY_SIZE];
type MatchData = [u8; MATCH_DATA_SIZE];
type MatchHash = [u8; MATCH_HASH_SIZE];

// namecoin name 
struct Player {
    pubkey: Pubkey,
    bux: u32,
}

// TODO: trait for crypto functions e.g. signing verifying
struct Referee {
    pubkey: Pubkey,
}

struct Signature([u8; SIGNATURE_SIZE]);

struct MatchResult {
    players: Vec<Player>,
    signatures: Vec<Signature>,
    match_data: MatchData,
    match_hash: MatchHash,
}

struct Match {
    players: Vec<Player>,
}

struct Tournament {
    players: Vec<Player>,

}

struct MultisigEscrow {
    pubkey: Pubkey, 
    player_keys: Vec<Pubkey>,
    referee_keys: Vec<Pubkey>,
}

impl MultisigEscrow {
    pub fn new(player_keys: Vec<Pubkey>, referee_keys: Vec<Pubkey>, bux: Vec<u32>) -> Self {
        MultisigEscrow {
            pubkey: MultisigEscrow::make_pubkey(&player_keys, &referee_keys),
            player_keys: player_keys,
            referee_keys: referee_keys,
        }

    }

    fn make_pubkey(player_keys: &Vec<Pubkey>, referee_keys: &Vec<Pubkey>) -> Pubkey {
        [0;PUBKEY_SIZE]
    }

    fn deposit() {

    }

    fn withdraw() {

    }
}
