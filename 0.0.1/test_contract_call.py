import asyncio
import base58
from py_near.account import Account

# These should be replaced with actual values in production
master_account_id = "YOUR_AGENT_ACCOUNT_ID"  # Replace with your agent account ID
master_private_key = "YOUR_AGENT_PRIVATE_KEY"  # Replace with your agent private key
contract_id = "dbrd.near"

async def test_contract_call():
    """Test function to verify contract communication"""
    try:
        # Create an account instance
        acc = Account(master_account_id, master_private_key)
        print(f"Created account instance for: {master_account_id}")
        
        # Create a test data_id with all zeros
        test_data_id = [0] * 32
        
        # Convert to base58 string
        test_data_id_str = base58.b58encode(bytes(test_data_id)).decode('utf-8')
        
        # Prepare test args
        test_args = {
            "data_id": test_data_id_str,
            "response": "This is a test response from the agent"
        }
        
        print(f"Using test args: {test_args}")
        
        # Call the contract
        tr = await acc.function_call(
            contract_id,
            'agent_response',
            test_args,
            300000000000000,
            0
        )
        print(f"Test call successful: {tr.transaction.hash}")
        print(f"View transaction at: https://nearblocks.io/txns/{tr.transaction.hash}")
    except Exception as e:
        print(f"Test call failed: {str(e)}")
        import traceback
        print(f"Traceback: {traceback.format_exc()}")

async def simple_test():
    """Test with simpler method"""
    try:
        # Create an account instance
        acc = Account(master_account_id, master_private_key)
        print(f"Created account instance for: {master_account_id}")
        
        # Simple args
        simple_args = {
            "response": "This is a simple test from the agent"
        }
        
        print(f"Using simple args: {simple_args}")
        
        # Call a simpler contract method
        tr = await acc.function_call(
            contract_id,
            'simple_agent_response',
            simple_args,
            300000000000000,
            0
        )
        print(f"Simple test call successful: {tr.transaction.hash}")
        print(f"View transaction at: https://nearblocks.io/txns/{tr.transaction.hash}")
    except Exception as e:
        print(f"Simple test call failed: {str(e)}")
        import traceback
        print(f"Traceback: {traceback.format_exc()}")

# Run the tests
async def main():
    print("Running contract tests...")
    print("1. Testing agent_response function:")
    await test_contract_call()
    
    print("\n2. Testing simple_agent_response function:")
    await simple_test()

if __name__ == "__main__":
    asyncio.run(main()) 