# Notes

## key fixes
- recreate repo and re-sync to github

anchor init fresh-escrow
cd fresh-escrow


- space = 8 + std::mem::size_of::<Escrow>(),
- arbitrator key derivation and hardcoding


- have grok double check my contract
- have grok check the tests for the whole contract


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