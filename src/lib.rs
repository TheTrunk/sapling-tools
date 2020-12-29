use json::object;
use std::path::Path;
use wasm_bindgen::prelude::*;
use zcash_client_backend::encoding::{
    decode_extended_full_viewing_key, decode_payment_address, decode_transparent_address,
    encode_extended_full_viewing_key, encode_extended_spending_key, encode_payment_address,
};
use zcash_primitives::{
    consensus::{BlockHeight, BranchId, MAIN_NETWORK},
    legacy::Script,
    transaction::{
        builder::Builder,
        components::{Amount, OutPoint, TxOut},
    },
    zip32::{ChildIndex, ExtendedFullViewingKey, ExtendedSpendingKey},
};
use zcash_proofs::prover::LocalTxProver;
extern crate hex;
extern crate console_error_panic_hook;
use hex::FromHex;
use secp256k1::key::SecretKey;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct TransparentUtxos {
    pub private_key: String,
    pub utxo: String,
    pub vout_index: u32,
    pub amount: u64,
    pub script: String,
}

impl TransparentUtxos {
    pub fn new(private_key: String, utxo: String, vout_index: u32, amount: u64, script: String) -> Self {
        Self { private_key, utxo, vout_index, amount, script }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TransparentRecipients {
    pub address: String,
    pub amount: u64,
}

impl TransparentRecipients {
    pub fn new(address: String, amount: u64, ) -> Self {
        Self { address, amount }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SaplingRecipients {
    pub view_key: String,
    pub address: String,
    pub amount: u64
}

impl SaplingRecipients {
    pub fn new(view_key: String, address: String, amount: u64, ) -> Self {
        Self { view_key, address, amount }
    }
}

fn generate_zaddr(
    seed: &[u8],
    bip: u32,
    slip: u32,
    index: u32,
    spending: String,
    viewing: String,
    address: String,
) -> (String, String, String) {
    let spk: ExtendedSpendingKey = ExtendedSpendingKey::from_path(
        &ExtendedSpendingKey::master(seed),
        &[
            ChildIndex::Hardened(bip),
            ChildIndex::Hardened(slip),
            ChildIndex::Hardened(index),
        ],
    );

    let encoded_pk = encode_extended_spending_key(&spending, &spk);

    let extfvk = ExtendedFullViewingKey::from(&spk);
    let encoded_vk = encode_extended_full_viewing_key(&viewing, &extfvk);

    let addr = &extfvk.default_address().unwrap().1;
    let encoded = encode_payment_address(&address, &addr);

    return (encoded, encoded_pk, encoded_vk);
}

#[wasm_bindgen]
pub fn get_zaddr(
    seed: String,
    bip: u32,
    slip44: u32,
    index: u32,
    spending: String,
    viewing: String,
    address: String,
) -> String {
    let (addr, pk, vk) = generate_zaddr(
        seed.as_bytes(),
        bip,
        slip44,
        index,
        spending,
        viewing,
        address,
    );
    let derived_address = object! {
        "address"       => addr,
        "private_key"   => pk,
        "viewing_key"   => vk
    };
    return json::stringify_pretty(derived_address, 2);
}

pub const B58_PUBKEY_ADDRESS_PREFIX_ZCASH: [u8; 2] = [0x1c, 0xb8];
pub const B58_SCRIPT_ADDRESS_PREFIX_ZCASH: [u8; 2] = [0x1c, 0xbd];
pub const HRP_SAPLING_PAYMENT_ADDRESS: &str = "zs";
pub const HRP_SAPLING_EXTENDED_SPENDING_KEY: &str = "secret-extended-key-main";
pub const HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY: &str = "zxviews";

// utxo array, recipient array, height, network, fee (not possible)
// utxo has private key unlocking it, txid, voutindex, amount, script
pub fn complex_send(
    transparent_utxos: Vec<TransparentUtxos>,
    transparent_recipients: Vec<TransparentRecipients>,
    sapling_recipients: Vec<SaplingRecipients>,
    height: u32,
    coin: String,
    sapling_spend: String,
    sapling_output: String,
) -> String {
    let mut pubkey_prefix: [u8; 2] = [0x1c, 0xb8];
    let mut script_prefix: [u8; 2] = [0x1c, 0xbd];
    let mut branch = BranchId::Canopy;
    let mut hrp_address = HRP_SAPLING_PAYMENT_ADDRESS;
    let mut hrp_view = HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY;
    if coin == "zec" {
        pubkey_prefix = B58_PUBKEY_ADDRESS_PREFIX_ZCASH;
        script_prefix = B58_SCRIPT_ADDRESS_PREFIX_ZCASH;
        branch = BranchId::Canopy;
        hrp_address = HRP_SAPLING_PAYMENT_ADDRESS;
        hrp_view = HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY;
    }
    if coin == "zel" {
        pubkey_prefix = B58_PUBKEY_ADDRESS_PREFIX_ZCASH;
        script_prefix = B58_SCRIPT_ADDRESS_PREFIX_ZCASH;
        branch = BranchId::Sapling;
        hrp_address = "za";
        hrp_view = "zxviewa";
    }

    let tx_prover = LocalTxProver::new(
        Path::new(&sapling_spend),
        Path::new(&sapling_output),
    );

    let mut builder = Builder::new(MAIN_NETWORK.clone(), BlockHeight::from_u32(height));

    for utxo in transparent_utxos { // (private_key, utxo, vout_index, amount, script)
        let priv_key_array = Vec::from_hex(utxo.private_key).expect("Decoding failed");
        let non_wallet_sk = SecretKey::from_slice(priv_key_array.as_slice()).unwrap();
        // let secp = Secp256k1::new();
        // let non_wallet_pk = PublicKey::from_secret_key(&secp, &non_wallet_sk);

        let mut txid: [u8; 32] = <[u8; 32]>::from_hex(utxo.utxo).expect("Decoding failed");
        txid.reverse();
        let outpoint = OutPoint::new(txid, utxo.vout_index);
        let utxo_script_vec = Vec::from_hex(utxo.script).expect("Decoding failed");

        let coin = TxOut {
            value: Amount::from_u64(utxo.amount).unwrap(),
            script_pubkey: Script { 0: utxo_script_vec },
        };
        builder
            .add_transparent_input(non_wallet_sk, outpoint.clone(), coin.clone())
            .expect("Input error");
    }

    for t_recipient in transparent_recipients {
        let public_key_hash =
            match decode_transparent_address(&pubkey_prefix, &script_prefix, &t_recipient.address) {
                Ok(public_key_hash) => public_key_hash.unwrap(),
                Err(e) => {
                    let e = format!("Error decoding address: {:?}", e);
                    return Err(e).expect("Address decode failed");
                }
            };
        builder
            .add_transparent_output(&public_key_hash, Amount::from_u64(t_recipient.amount).unwrap())
            .expect("Transparent Output error");
    }

    for s_recipient in sapling_recipients {
        let decoded_vk = match decode_extended_full_viewing_key(hrp_view, &s_recipient.view_key) {
            Ok(decoded_vk) => decoded_vk.unwrap(),
            Err(e) => {
                let e = format!("Error decoding extended full viewing key: {:?}", e);
                return Err(e).expect("Decoding VK failed");
            }
        };
        let ovk = decoded_vk.fvk.ovk;

        let sapling_addr = match decode_payment_address(&hrp_address, &s_recipient.address) {
            Ok(sapling_addr) => sapling_addr.unwrap(),
            Err(e) => {
                let e = format!("Error decoding sapling address: {:?}", e);
                return Err(e).expect("Sapling address decode failed");
            }
        };

        builder
            .add_sapling_output(
                Some(ovk),
                sapling_addr,
                Amount::from_u64(s_recipient.amount).unwrap(),
                None,
            )
            .expect("Sapling Output error");
    }

    let (tx, _tx_metadata) = match builder.build(branch, &tx_prover) {
        Ok(res) => res,
        Err(e) => {
            let e = format!("Error creating transaction: {:?}", e);
            return Err(e).expect("Construction failed");
        }
    };

    let mut tx_id = tx.txid().0.to_vec();
    tx_id.reverse();
    let tx_id_hex = hex::encode(tx_id);

    println!("Transaction ID: {:?}", tx_id_hex);
    // Create the TX bytes
    let mut raw_tx = vec![];
    tx.write(&mut raw_tx).unwrap();

    let result = hex::encode(raw_tx);

    return result;
}

#[wasm_bindgen]
pub fn send(
    transparent_utxos: &JsValue,
    transparent_recipients: &JsValue,
    sapling_recipients: &JsValue,
    height: u32,
    coin: String,
    sapling_spend: String,
    sapling_output: String,
) -> String {
    console_error_panic_hook::set_once();
    let t_utxos: Vec<TransparentUtxos> = transparent_utxos.into_serde().unwrap();
    let t_recipients: Vec<TransparentRecipients> = transparent_recipients.into_serde().unwrap();
    let s_recipients: Vec<SaplingRecipients> = sapling_recipients.into_serde().unwrap();
    let tx = complex_send(
        t_utxos,
        t_recipients,
        s_recipients,
        height,
        coin,
        sapling_spend,
        sapling_output,
    );
    return tx;
}