use serde::{
    Deserialize,
    Serialize,
};

use tglib::{
    bdk::bitcoin::{
        blockdata::transaction::OutPoint,
        hashes::{
            Hash as BitcoinHash,
            HashEngine,
            sha256::Hash as ShaHash,
            sha256::HashEngine as ShaHashEngine,
        },
        secp256k1::Signature,
        util::psbt::Input,
        Address,
        PublicKey,
    },
    player::PlayerName,
    payout::PayoutRecord,
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContract {
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SendContractBody {
    pub contract: EventContract,
    pub player_name: PlayerName,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendPayoutBody {
    pub payout: PayoutRecord,
    pub player_name: PlayerName,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthTokenSig {
    pub player_name: PlayerName,
    pub pubkey: PublicKey,
    pub sig_hex: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerContractInfo {
    pub name: PlayerName,
    pub escrow_pubkey: PublicKey,
    pub change_address: Address,
    pub payout_address: Address,
    pub utxos: Vec<(OutPoint, u64, Input)>,
}

impl PlayerContractInfo {
    pub fn hash(&self) -> Vec<u8> {
        let mut engine = ShaHashEngine::default();
        engine.input(self.name.0.as_bytes());
        engine.input(&self.escrow_pubkey.to_bytes());
        engine.input(&self.change_address.to_string().as_bytes());
        for (outpoint, _, _) in self.utxos.clone() {
            engine.input(outpoint.txid.as_inner());
            engine.input(&Vec::from(outpoint.vout.to_be_bytes()));
        }

        let hash: &[u8] = &ShaHash::from_engine(engine);
        Vec::from(hash)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetContractInfoBody {
    pub contract_info: PlayerContractInfo,
    pub pubkey: PublicKey,
    pub sig_hex: String,
}

pub trait ExchangeService {
    fn set_contract_info(&self, info: PlayerContractInfo, pubkey: PublicKey, sig: Signature) -> Result<()>;
    fn get_contract_info(&self, player_name: PlayerName) -> Result<Option<PlayerContractInfo>>;
    fn send_contract(&self, contract: EventContract, player_name: PlayerName) -> Result<()>;
    fn send_payout(&self, payout: PayoutRecord, player_name: PlayerName) -> Result<()>;
    fn get_auth_token(&self, player_name: &PlayerName) -> Result<Vec<u8>>;
    fn receive_contract(&self, auth: AuthTokenSig) -> Result<Option<EventContract>>;
    fn receive_payout(&self, auth: AuthTokenSig) -> Result<Option<PayoutRecord>>;
}
