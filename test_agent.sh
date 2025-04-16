#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}   NEAR AI Agent Communication Tester    ${NC}"
echo -e "${BLUE}=========================================${NC}"

# Check if account ID is provided
if [ -z "$1" ]; then
  echo -e "${RED}Error: Please provide your NEAR account ID as the first argument${NC}"
  echo -e "Usage: ./test_agent.sh YOUR_ACCOUNT.near"
  exit 1
fi

ACCOUNT_ID=$1
CONTRACT_ID="dbread.near"
NETWORK="mainnet"

echo -e "${YELLOW}Testing with:${NC}"
echo -e "  Account ID: ${GREEN}$ACCOUNT_ID${NC}"
echo -e "  Contract: ${GREEN}$CONTRACT_ID${NC}"
echo -e "  Network: ${GREEN}$NETWORK${NC}"
echo

# Test 1: Simple test with direct_llm_query
echo -e "${BLUE}Test 1: Direct LLM Query${NC}"
echo -e "${YELLOW}This tests if the contract works with a direct response without using the agent${NC}"
near call $CONTRACT_ID direct_llm_query '{"prompt": "test query"}' --accountId $ACCOUNT_ID --gas 300000000000000 --networkId $NETWORK

echo
echo -e "${BLUE}=========================================${NC}"

# Test 2: Try query_agent_async
echo -e "${BLUE}Test 2: Asynchronous Agent Query${NC}"
echo -e "${YELLOW}This will emit an event to the agent and return a request ID${NC}"
RESPONSE=$(near call $CONTRACT_ID query_agent_async '{"prompt": "test async query"}' --accountId $ACCOUNT_ID --gas 300000000000000 --networkId $NETWORK)
echo "$RESPONSE"

# Extract request_id from the response
REQUEST_ID=$(echo "$RESPONSE" | grep -o '"request_id": "[^"]*' | sed 's/"request_id": "//')

if [ -n "$REQUEST_ID" ]; then
  echo
  echo -e "${GREEN}Got request ID: $REQUEST_ID${NC}"
  echo -e "${YELLOW}Waiting 15 seconds for agent to process...${NC}"
  sleep 15

  # Check if there's a response
  echo
  echo -e "${BLUE}Checking for response...${NC}"
  near view $CONTRACT_ID get_response_by_id '{"base58_id": "'$REQUEST_ID'"}' --networkId $NETWORK
else
  echo -e "${RED}Failed to get request ID${NC}"
fi

echo
echo -e "${BLUE}=========================================${NC}"

# Test 3: Try direct_query
echo -e "${BLUE}Test 3: Direct Query${NC}"
echo -e "${YELLOW}This tests another way to send queries to the agent${NC}"
near call $CONTRACT_ID direct_query '{"prompt": "test direct query"}' --accountId $ACCOUNT_ID --gas 300000000000000 --networkId $NETWORK

echo
echo -e "${BLUE}=========================================${NC}"
echo -e "${GREEN}Testing complete!${NC}"
echo
echo -e "${YELLOW}If all tests returned 'PENDING' or no response was found, the issue is likely with:${NC}"
echo -e "1. Agent environment variables (master_account_id and master_private_key)"
echo -e "2. Agent not receiving events or not able to respond"
echo -e "3. Agent permissions to call the contract"
echo
echo -e "${YELLOW}Next steps:${NC}"
echo -e "1. Send a 'health check' message to your agent in NEAR AI Hub"
echo -e "2. Check agent logs in NEAR AI Hub for detailed error messages"
echo -e "3. Verify that your environment variables are correctly set" 