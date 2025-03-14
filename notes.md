# Notes

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