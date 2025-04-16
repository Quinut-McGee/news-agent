# NEAR AI Hub Agent Contract

This smart contract allows interaction with agents deployed on the NEAR AI Hub. It provides a simple interface to send prompts to your agent and receive responses.

## Features

- Send prompts to your NEAR AI Hub agent
- Receive and store agent responses
- Query stored responses by request ID
- Secure agent response validation

## Contract Interface

### Initialization

```rust
pub fn new(agent_account_id: AccountId, agent_name: String) -> Self
```

Initialize the contract with your agent's account ID and name.

### Methods

1. **Query Agent**
```rust
pub fn query_agent(&mut self, prompt: String) -> PromiseOrValue<String>
```
Send a prompt to your agent and receive a response.

2. **Get Response**
```rust
pub fn get_response(&self, request_id: CryptoHash) -> Option<AgentResponse>
```
Retrieve a stored response by its request ID.

## Usage Example

1. Deploy the contract:
```bash
near deploy --wasmFile target/wasm32-unknown-unknown/release/near_ai_agent.wasm --accountId your-contract.near
```

2. Initialize the contract:
```bash
near call your-contract.near new '{"agent_account_id": "your-agent.near", "agent_name": "your-agent-name"}' --accountId your-contract.near
```

3. Send a prompt to your agent:
```bash
near call your-contract.near query_agent '{"prompt": "Your prompt here"}' --accountId your-account.near
```

4. Get a stored response:
```bash
near view your-contract.near get_response '{"request_id": "your-request-id"}'
```

## Security

- Only the specified agent account can send responses
- Responses are stored securely in the contract's state
- All interactions are logged for transparency

## Building

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Testing

```bash
cargo test
``` 