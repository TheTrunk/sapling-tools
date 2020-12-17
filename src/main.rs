pub mod lib;

pub fn main() {
    let x = lib::get_zaddr("myamazingseedphrase".to_string(), 32, 133, 0, "secret-extended-key-main".to_string(), "zxviews".to_string(), "zs".to_string()); // seed, bip, slip, index, extendedSpendingKey prefix, extendedViewKey prefix, addressPrefix
    println!("{}", x);
}