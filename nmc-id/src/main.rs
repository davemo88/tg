use warp::{
    Filter,
    Reply,
    Rejection,
};
use serde_json;
use tglib::{
    bdk::{
        bitcoin::{
            PublicKey,
            secp256k1::Signature,
        },
        blockchain::noop_progress,
        electrum_client::Client,
    },
    bip39::Mnemonic,
    contract::PlayerContractInfo,
    player::{
        PlayerId,
        PlayerIdService,
        PlayerName,
        PlayerNameService,
    },
    wallet::{
        EscrowWallet,
        SigningWallet,
    },
    mock::{
        Trezor,
        ELECTRS_SERVER,
        PLAYER_2_MNEMONIC,
        NETWORK,
    }
};
use player_wallet::wallet::PlayerWallet;

type WebResult<T> = std::result::Result<T, Rejection>;

#[derive(Clone)]
struct NmcId;

impl NmcId {
    pub fn new() -> Self {
        NmcId
    }
}

impl PlayerNameService for NmcId {
    fn get_player_name(&self, pubkey: &PublicKey) -> Option<PlayerName> {
        None
    }

    fn get_contract_info(&self, name: PlayerName) -> Option<PlayerContractInfo> {
        None
    }
    fn set_contract_info(&self, name: PlayerName, info: PlayerContractInfo, sig: Signature) -> Option<PlayerContractInfo> {
        None
    }
    fn register_name(&self, name: PlayerName, pubkey: &PublicKey, sig: Signature) -> Result<(), String> {
        Err("oh no".to_string())
    }
}

impl PlayerIdService for NmcId {
    fn get_player_id(&self, _pubkey: &PublicKey) -> Option<PlayerId> {
        None
    }

    fn get_player_info(&self, player_id: PlayerId) -> Option<PlayerContractInfo> {
        println!("request for info on player {}", player_id.0);
        let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap());
        let client = Client::new(ELECTRS_SERVER).unwrap();
        let player_wallet = PlayerWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK, client);
        let escrow_pubkey = EscrowWallet::get_escrow_pubkey(&player_wallet);
        player_wallet.wallet.sync(noop_progress(), None).unwrap();
        Some(PlayerContractInfo {
            escrow_pubkey,
            change_address: player_wallet.wallet.get_new_address().unwrap(),
            utxos: player_wallet.wallet.list_unspent().unwrap(),
        })
    }

// TODO this endpoint needs auth requiring ownership of the corresponding player_id
// e.g. if the player_id is a namecoin name, then a signature of some random data
// by the owner of the name will do nicely
//    fn set_player_info(&self, player_id: PlayerId, info: PlayerContractInfo) -> Result<()> {
//        Err(TgError("invalid signature"))
//    }
}

async fn id_handler(_pubkey: String, _nmcid: NmcId) -> WebResult<impl Reply>{
    Ok("not implemented".to_string())
}

async fn info_handler(player_id: String, nmcid: NmcId) -> WebResult<impl Reply>{
    let info = nmcid.get_player_info(PlayerId(player_id)).unwrap();
    Ok(serde_json::to_string(&info).unwrap())
}

#[tokio::main]
async fn main() {
    
    let nmc_id = warp::any().map(move || NmcId::new());

    let get_player_id = warp::path("get-player-id")
        .and(warp::path::param::<String>())
        .and(nmc_id.clone())
        .and_then(id_handler);

    let get_player_info = warp::path("get-player-info")
        .and(warp::path::param::<String>())
        .and(nmc_id.clone())
        .and_then(info_handler);

    let routes = get_player_id.or(get_player_info);

    warp::serve(routes).run(([0, 0, 0, 0], 18420)).await;
}
