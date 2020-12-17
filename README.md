# Sapling Tools

This module is build around https://github.com/zcash/librustzcash
Is meannt to provide some basic needed functionality to Zcash Sapling cryptography such as

- Address Generation

## 🚴 Usage
```
npm i sapling-tools
const saplingTools = require('sapling-tools');
const derivedAddress = await saplingTools.get_zaddr("myamazingseedphrase", 32, 133, 0, "secret-extended-key-main", "zxviews", "zs");
const address = derivedAddress.address;
const privateKey = derivedAddress.private_key;
const viewingKey = derivedAddress.viewing_key;
```

### 🛠️ Build with `wasm-pack build`

```
wasm-pack build --target bundler --release --out-name index
```
