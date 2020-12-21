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
use hex::FromHex;
use secp256k1::key::SecretKey;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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
    transparent_utxos: Vec<(&str, &str, u32, u64, &str)>,
    transparent_recipients: Vec<(&str, u64)>,
    sapling_recipients: Vec<(&str, &str, u64)>,
    height: u32,
    coin: &str,
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
        Path::new("C:/Users/User/AppData/Roaming/ZcashParams/sapling-spend.params"),
        Path::new("C:/Users/User/AppData/Roaming/ZcashParams/sapling-output.params"),
    );

    let mut builder = Builder::new(MAIN_NETWORK.clone(), BlockHeight::from_u32(height));

    for (private_key, utxo, vout_index, amount, script) in transparent_utxos {
        let priv_key_array = Vec::from_hex(private_key).expect("Decoding failed");
        let non_wallet_sk = SecretKey::from_slice(priv_key_array.as_slice()).unwrap();
        // let secp = Secp256k1::new();
        // let non_wallet_pk = PublicKey::from_secret_key(&secp, &non_wallet_sk);

        let mut txid: [u8; 32] = <[u8; 32]>::from_hex(utxo).expect("Decoding failed");
        txid.reverse();
        let outpoint = OutPoint::new(txid, vout_index);
        let utxo_script_vec = Vec::from_hex(script).expect("Decoding failed");

        let coin = TxOut {
            value: Amount::from_u64(amount).unwrap(),
            script_pubkey: Script { 0: utxo_script_vec },
        };
        builder
            .add_transparent_input(non_wallet_sk, outpoint.clone(), coin.clone())
            .expect("Input error");
    }

    for (address, amount) in transparent_recipients {
        let public_key_hash =
            match decode_transparent_address(&pubkey_prefix, &script_prefix, address) {
                Ok(public_key_hash) => public_key_hash.unwrap(),
                Err(e) => {
                    let e = format!("Error decoding address: {:?}", e);
                    return Err(e).expect("Address decode failed");
                }
            };
        builder
            .add_transparent_output(&public_key_hash, Amount::from_u64(amount).unwrap())
            .expect("Transparent Output error");
    }

    for (view_key, address, amount) in sapling_recipients {
        let decoded_vk = match decode_extended_full_viewing_key(hrp_view, &view_key) {
            Ok(decoded_vk) => decoded_vk.unwrap(),
            Err(e) => {
                let e = format!("Error decoding extended full viewing key: {:?}", e);
                return Err(e).expect("Decoding VK failed");
            }
        };
        let ovk = decoded_vk.fvk.ovk;

        let sapling_addr = match decode_payment_address(&hrp_address, address) {
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
                Amount::from_u64(amount).unwrap(),
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
