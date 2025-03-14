# Notes


Next Steps
Once this succeeds, we’ll add back fields incrementally, starting with simple ones like fee: u64 and fiat_deadline: i64, adjusting space each time (e.g., 165 + 8 = 173, then 173 + 8 = 181), and testing after each step. This will help us pinpoint where the deserialization breaks in the full version.
Run this and let me know the results! We’ll build from this solid foundation.





space = 8 + std::mem::size_of::<Escrow>(),

## Build, Deploy, Test Routine
anchor clean
cargo clean
mkdir -p target/deploy
cp keys/program-devnet/program-keypair.json target/deploy/simple_escrow-keypair.json
rm -rf test-ledger/*
<!-- stop validator -->
anchor build
solana-test-validator
anchor deploy
anchor test --skip-local-validator