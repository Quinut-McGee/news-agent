# NEAR AI Agent Smart Contract Integration

This project demonstrates how to integrate a NEAR AI Hub agent with a smart contract, allowing users to query an AI agent through a NEAR contract.

## Project Structure

```
.
├── 0.0.1/                 # NEAR AI Hub agent code
│   ├── agent.py           # Agent implementation
│   └── metadata.json      # Agent configuration
├── smart-contract/        # NEAR smart contract
│   ├── src/               # Contract source code
│   ├── Cargo.toml         # Rust dependencies
│   └── README.md          # Contract documentation
├── test_agent.sh          # Testing script
└── env_setup.md           # Environment setup guide
```

## Getting Started

1. **Set up the environment variables for the agent**
   - Follow the instructions in `env_setup.md`

2. **Deploy the agent to NEAR AI Hub**
   - Upload the agent files from the `0.0.1` directory to NEAR AI Hub

3. **Deploy the smart contract** (if not already deployed)
   - Build and deploy the smart contract to your NEAR account

## Usage

### Query the Agent

The smart contract provides several methods to interact with the AI agent:

**Asynchronous Query (Recommended):**
```bash
near call dbread.near query_agent_async '{"prompt": "your question here"}' --accountId YOUR_ACCOUNT.near --gas 300000000000000 --networkId mainnet
```

This returns a request ID which you can use to check for the response:

```bash
near view dbread.near get_response_by_id '{"base58_id": "YOUR_REQUEST_ID"}' --networkId mainnet
```

### Website Integration

To integrate with a website:

1. Use `query_agent_async` to send a query and get a request ID
2. Poll `get_response_by_id` until you get a response (not "PENDING")

Example JavaScript:
```javascript
// Send query
const result = await near.call(contractId, 'query_agent_async', { prompt: userQuestion });
const requestId = JSON.parse(result).request_id;

// Poll for response
function checkResponse() {
  near.view(contractId, 'get_response_by_id', { base58_id: requestId })
    .then(response => {
      if (response && response !== 'PENDING') {
        // Handle response
        displayResponse(response);
      } else {
        // Check again after a delay
        setTimeout(checkResponse, 3000);
      }
    });
}

checkResponse();
```

## Troubleshooting

If you're having issues with the agent not responding:

1. **Run the test script:**
   ```bash
   ./test_agent.sh YOUR_ACCOUNT.near
   ```

2. **Check agent health:**
   Send a "health check" message to your agent in NEAR AI Hub

3. **Run event diagnostics:**
   Send "event diagnostic" to your agent to test event handling

## License

This project is licensed under the MIT License. 