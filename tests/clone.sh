#!/bin/bash

if [ $# -gt 0 ]; then
  if [ $1 == "clone" ]; then
    # LP: H5vPY967v8DkZRaZVNxDaMrHUdtovRUET8c6AXo3BirF https://raydium.io/liquidity/increase/?mode=add&pool_id=9LfXeYQgTXJWhyTQhykCSnfUDd1ffCYA1LcSdcwaRLBk
    solana program dump -u m 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8 raydium.so
    solana account -u m H5vPY967v8DkZRaZVNxDaMrHUdtovRUET8c6AXo3BirF --output-file lp.json --output json-compact
    # modify below to set new owner
    solana account -u m 1R8BFjYJYCTgifSwTyPA7gr6HhYPsCr9HHMdXvVLGhm --output-file lp_ata.json --output json-compact
    solana account -u m 9LfXeYQgTXJWhyTQhykCSnfUDd1ffCYA1LcSdcwaRLBk --output-file amm.json --output json-compact
    # owner of lp-ata 722d254xnMZ6Y8vWcpYbkYtJSg8JeGNhfyc7EKtnjt3R
    exit 0;
  fi

  if [ $1 == "patch" ]; then
    npx tsx tests/deserializer.ts
  fi

  if [ $1 == "start" ]; then
    json_data=$(cat tests/lp_ata_new_owner.json)
    lp_ata_pubkey=$(echo "$json_data" | jq -r '.pubkey')

    solana-test-validator -r --bpf-program 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8 tests/raydium.so \
    --account H5vPY967v8DkZRaZVNxDaMrHUdtovRUET8c6AXo3BirF tests/lp.json \
    --account $lp_ata_pubkey tests/lp_ata_new_owner.json \
    --account 9LfXeYQgTXJWhyTQhykCSnfUDd1ffCYA1LcSdcwaRLBk tests/amm.json 
        
    exit 0;
  fi
fi

if [ $# -eq 0 ]; then
  echo "No arguments supplied"
  echo "Usage: $0 [options] \n"
  echo "   clone.sh clone : Clone accounts from solana mainnet pool\n"
  echo "   clone.sh patch : Patch the deserialized data and add owner from ~/.config/solana/id.json\n"
  echo "   clone.sh start : Start the solana test validator with the cloned accounts\n"
fi






