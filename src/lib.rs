use wasm_bindgen::prelude::*;
use json::{object};
use zcash_primitives::zip32::{ChildIndex, ExtendedSpendingKey, ExtendedFullViewingKey};
use zcash_client_backend::encoding::{encode_extended_spending_key, encode_extended_full_viewing_key, encode_payment_address};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, myproject!");
}

fn generate_zaddr(seed: &[u8], bip: u32, slip: u32, index: u32, spending: String, viewing: String, address: String) -> (String, String, String) {
    let spk: ExtendedSpendingKey = ExtendedSpendingKey::from_path(
             &ExtendedSpendingKey::master(seed),
             &[
                 ChildIndex::Hardened(bip),
                 ChildIndex::Hardened(slip),
                 ChildIndex::Hardened(index)
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
pub fn get_zaddr(seed: String, bip: u32, slip44: u32, index: u32, spending: String, viewing: String, address: String) -> String {
    let (addr, pk, vk) = generate_zaddr(seed.as_bytes(), bip, slip44, index, spending, viewing, address);
    let derived_address = object!{
        "address"       => addr,
        "private_key"   => pk,
        "viewing_key"   => vk
    };
    return json::stringify_pretty(derived_address, 2);
}
