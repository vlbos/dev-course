License: MIT-0

```shell
# unittest with runtime-benchmarks feature
 cargo test --package pallet-kitties  --features runtime-benchmarks

# compile with runtime-benchmarks feature
cargo build --profile=production --features runtime-benchmarks

# benchmark dispatchables in kitties pallet
# download frame-weight-template.hbs from [polkadot-sdk repo](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/.maintain/frame-weight-template.hbs).
./target/production/solochain-template-node benchmark pallet \
--wasm-execution=compiled \
--pallet pallet_kitties \
--extrinsic "*" \
--steps 20 \
--repeat 10 \
--output pallets/kitties/src/weights.rs \
--template .maintain/frame-weight-template.hbs
```
