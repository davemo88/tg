use std::{
    str::FromStr,
    thread::sleep,
    time::Duration,
};
use redis::{
    self,
    Commands,
    RedisResult,
    Connection as SyncConnection,
};
use warp::{
    Filter,
    Reply,
    Rejection,
};
use tglib::{
    bdk::{
        bitcoin::{
            Address,
            PublicKey,
            consensus,
            util::{
                bip32::{
                    ExtendedPubKey,
                    DerivationPath,
                    Fingerprint,
                },
                psbt::PartiallySignedTransaction,
            },
            secp256k1::{
                Secp256k1,
                Signature,
            }
        },
        blockchain::ElectrumBlockchain,
        database::MemoryDatabase,
        electrum_client::Client,
    },
    hex,
    Result,
    TgError,
    arbiter::ArbiterService,
    contract::Contract,
    payout::Payout,
    wallet::EscrowWallet,
    mock::{
        ARBITER_FINGERPRINT,
        ARBITER_XPUBKEY,
        ELECTRS_SERVER,
        NETWORK,
        REDIS_SERVER,
    },
};
mod wallet;
use wallet::Wallet;

type WebResult<T> = std::result::Result<T, Rejection>;

fn wallet() -> Wallet<ElectrumBlockchain, MemoryDatabase> {
    let mut client = Client::new(ELECTRS_SERVER);
    while client.is_err() {
        sleep(Duration::from_secs(1));
        client = Client::new(ELECTRS_SERVER);
    }
    Wallet::<ElectrumBlockchain, MemoryDatabase>::new(Fingerprint::from_str(ARBITER_FINGERPRINT).unwrap(), ExtendedPubKey::from_str(ARBITER_XPUBKEY).unwrap(), ElectrumBlockchain::from(client.unwrap()), NETWORK).unwrap()
}

#[derive(Clone)]
struct RbtrPublic {
    redis_client: redis::Client,
}

impl RbtrPublic {
    pub fn new(redis_client: redis::Client) -> Self {
        RbtrPublic {
            redis_client,
        }
    }

    fn get_con(&self) -> SyncConnection {
        self.redis_client.get_connection().unwrap()
    }

    fn push_contract(&self, con: &mut SyncConnection, hex: &str) -> RedisResult<String> {
        con.rpush("contracts", hex)?;
        Ok(String::from(hex))
    }

    fn push_payout(&self, con: &mut SyncConnection, hex: &str) -> RedisResult<String> {
        con.rpush("payouts", hex)?;
        Ok(String::from(hex))
    }
}


impl ArbiterService for RbtrPublic {
    fn get_escrow_pubkey(&self) -> Result<PublicKey> {
        Ok(EscrowWallet::get_escrow_pubkey(&wallet()))
    }

    fn get_fee_address(&self) -> Result<Address> {
        let w = wallet();
        let a = w.xpubkey.derive_pub(&Secp256k1::new(), &DerivationPath::from_str("m/0/0").unwrap()).unwrap();
        Ok(Address::p2wpkh(&a.public_key, w.network).unwrap())
    }

    fn submit_contract(&self, contract: &Contract) -> Result<Signature> {
        if wallet().validate_contract(&contract).is_ok() {
            let mut con = self.get_con();
            let cxid = self.push_contract(&mut con, &hex::encode(contract.to_bytes())).unwrap();
            for _ in 1..15 as u32 {
                sleep(Duration::from_secs(1));
                let r: RedisResult<String> = con.get(hex::encode(contract.cxid()));
                if let Ok(sig) = r {
                    let _r : RedisResult<String> = con.del(cxid);
                    return Ok(Signature::from_compact(&hex::decode(sig).unwrap()).unwrap())
                }
            }
        }
        Err(TgError("invalid contract"))
    }

    fn submit_payout(&self, payout: &Payout) -> Result<PartiallySignedTransaction> {
        if wallet().validate_payout(&payout).is_ok() {
            println!("rbtr-public validated payout");
            let mut con = self.get_con();
            let _r = self.push_payout(&mut con, &hex::encode(payout.to_bytes())).unwrap();
            let cxid = hex::encode(payout.contract.cxid());
            for _ in 1..15 as u32 {
                sleep(Duration::from_secs(1));
                let r: RedisResult<String> = con.get(cxid.clone());
                if let Ok(tx) = r {
                    let _r : RedisResult<String> = con.del(cxid);
                    return Ok(consensus::deserialize::<PartiallySignedTransaction>(&hex::decode(tx).unwrap()).unwrap())
                }
            }
        }
        Err(TgError("invalid payout"))
    }
}

async fn contract_handler(contract_hex: String, rbtr: RbtrPublic) -> WebResult<impl Reply> {
    let contract = Contract::from_bytes(hex::decode(contract_hex.clone()).unwrap()).unwrap();
    if let Ok(sig) = rbtr.submit_contract(&contract) {
        Ok(hex::encode(sig.serialize_compact()))
    } else {
        Err(warp::reject())
    }
}

// TODO: somehow break out of serialize(decode( hell
async fn payout_handler(payout_hex: String, rbtr: RbtrPublic) -> WebResult<impl Reply> {
    let payout = Payout::from_bytes(hex::decode(payout_hex.clone()).unwrap()).unwrap();
    if let Ok(tx) = rbtr.submit_payout(&payout) {
        Ok(hex::encode(consensus::serialize(&tx)))
    } else {
        Err(warp::reject())
    }
}

fn redis_client() -> redis::Client {
    let mut client = redis::Client::open(REDIS_SERVER);
    while client.is_err() {
        sleep(Duration::from_secs(1));
        client = redis::Client::open(REDIS_SERVER);
    }
    client.unwrap()
}

#[tokio::main]
async fn main() {
    
    let rbtr_public = RbtrPublic::new(redis_client());
    let escrow_pubkey = rbtr_public.get_escrow_pubkey().unwrap();
    let fee_address = rbtr_public.get_fee_address().unwrap();
    let rbtr_public = warp::any().map(move || rbtr_public.clone());
    let escrow_pubkey = warp::any().map(move || escrow_pubkey.clone());
    let fee_address = warp::any().map(move || fee_address.clone());

    let get_escrow_pubkey = warp::path("escrow-pubkey")
        .and(escrow_pubkey)
        .map(|e: PublicKey | e.to_string()); 

    let get_fee_address = warp::path("fee-address")
        .and(fee_address)
        .map(|f: Address| f.to_string()); 

// TODO: can add validation filters for the string params in the following paths?
    let submit_contract = warp::path("submit-contract")
        .and(warp::path::param::<String>())
        .and(rbtr_public.clone())
        .and_then(contract_handler);

    let submit_payout = warp::path("submit-payout")
        .and(warp::path::param::<String>())
        .and(rbtr_public.clone())
        .and_then(payout_handler);

    let routes = get_escrow_pubkey
        .or(get_fee_address)
        .or(submit_contract)
        .or(submit_payout);

    warp::serve(routes).run(([0, 0, 0, 0], 5000)).await;
}
