#!/bin/bash
set -e

echo "Building the smart contract..."
cargo near build --no-locked

echo "Deploying the smart contract..."
near deploy dbread.near target/near/near_ai_agent.wasm --accountId dbread.near 