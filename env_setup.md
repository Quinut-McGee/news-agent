# Environment Setup Guide for NEAR AI Agent

This guide will help you set up the required environment variables in NEAR AI Hub for your agent to communicate properly with your smart contract.

## Required Environment Variables

Your agent needs these two environment variables:

1. `master_account_id`: Your NEAR account ID (e.g., `dbread.near`)
2. `master_private_key`: The full access key for your account

## How to Get Your Private Key

To get your full access key, you need to either:

1. Use the private key from when you created your account, or
2. Create a new full access key using NEAR CLI

### Using NEAR CLI:

```bash
# Log in to your account
near login

# Find your credentials file
cat ~/.near-credentials/mainnet/dbread.near.json
```

The output will show something like:
```json
{
  "account_id": "dbread.near",
  "public_key": "ed25519:XXXXXX",
  "private_key": "ed25519:XXXXXX"
}
```

Copy the `private_key` value - this is what you need for the `master_private_key` environment variable.

## Setting Environment Variables in NEAR AI Hub

1. Go to the NEAR AI Hub: https://nearai.fyi/
2. Navigate to your agent
3. Click on "Settings" or "Environment Variables"
4. Add both variables:
   - Key: `master_account_id`, Value: `dbread.near`
   - Key: `master_private_key`, Value: `ed25519:XXXXXXX`
5. Save the changes
6. **Important**: Re-deploy your agent after setting the environment variables

## Verification

To verify that your environment variables are correctly set:

1. Send a direct message to your agent with the text: "health check"
2. The agent will run a diagnostic test and show if it can access the environment variables and communicate with the contract

## Testing Contract Communication

After setting up the environment variables:

1. Run the included test script:
   ```bash
   ./test_agent.sh dbread.near
   ```

2. Check the agent logs in NEAR AI Hub to see detailed diagnostic information

## Common Issues

1. **Permission Denied**: Make sure your account has permission to call the contract
2. **Invalid Key Format**: The private key must include the prefix (`ed25519:`)
3. **Agent Not Processing Events**: Check agent logs for errors
4. **Contract Error**: Ensure the contract expects the parameters in the correct format 