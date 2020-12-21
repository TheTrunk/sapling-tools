pub mod lib;

pub fn main() {
    let x = lib::get_zaddr("myamazingseedphrase".to_string(), 32, 133, 0, "secret-extended-key-main".to_string(), "zxviewa".to_string(), "za".to_string()); // seed, bip, slip, index, extendedSpendingKey prefix, extendedViewKey prefix, addressPrefix
    println!("{}", x);

    let recipients: Vec<(&str, u64)> = vec![("t1UPSwfMYLe18ezbCqnR5QgdJGznzCUYHkj", 49997000)];
    let sapling_recipients: Vec<(&str, &str, u64)> = vec![("zxviewa1qveya6heqqqqpqy4cdpxtnq2nepcpu7j3zknymg8u04lsex9ttgjp0xz20drd5y38hy0dl52qhlx8gmzvkzv64uccxkjte5kgq5hyekthwjtwlj33an7d4asf8ywqlmrvp6tv6kr993tfq8ejrhenazxau3lk0qr4u7rz3yxd6fgw60hl4qnrsr3s3x0640cc90rx9czph8775sne3k9pyh0mklgqvaqvje3dhfhvs8k8zjj9wnf4556q7qh3pk6w8zucs3s53msw9q4qcvcx", "za1w02tz80epk77ud26080v6zt3svt3uu4gzv42mdl372uwdmusu2csmqrf6k57r7jmyyge7eenppx", 2000)]; // users viewving key for encrypting output, recipient sapling address, amount. TODO add memo
    let utxos: Vec<(&str, &str, u32, u64, &str)> = vec![("privkeyUnlockingUtxo", "ee3ad04c30569f9de2fefde2dcc0cd0eb28134c93337431d10ebb47d04e49895", 0, 50000000, "76a91473562bc6a1db9dc6effebc1ef4379942feb3cf2c88ac")]; // priv key, utxo txid, vout index, script
    let x = lib::complex_send(utxos, recipients, sapling_recipients, 756504, "zel"); 
    println!("{:?}", x);
} 