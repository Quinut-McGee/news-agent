import asyncio
import json
import base64
import base58
from nearai.agents.environment import Environment
from py_near.account import Account

master_account_id = globals()['env'].env_vars.get("master_account_id", None)
master_private_key = globals()['env'].env_vars.get("master_private_key", None)
contract_id = "chat.breadaily.near"  # Contract ID updated to match deployment

def bytes_to_str(bytes_arr):
    """Convert bytes array to base58 string"""
    return base58.b58encode(bytes(bytes_arr)).decode('utf-8')

def str_to_bytes(base58_str):
    """Convert base58 string back to bytes array"""
    return list(base58.b58decode(base58_str))

async def test_contract_call(env: Environment):
    """Test function to verify contract communication"""
    try:
        acc = Account(master_account_id, master_private_key)
        env.add_reply(f"Test - Using account: {master_account_id}")
        
        # Create a test data_id with all zeros
        test_data_id = [0] * 32
        
        # Convert to base58 string
        test_data_id_str = bytes_to_str(test_data_id)
        
        # Prepare test args as Python dictionary
        test_args = {
            "data_id": test_data_id_str,  # Pass as base58 string
            "response": "test response"
        }
        
        env.add_reply(f"Using test args: {test_args}")
        
        tr = await acc.function_call(
            contract_id,
            'agent_response',
            test_args,
            200000000000000,
            0
        )
        env.add_reply(f"Test call successful: {tr.transaction.hash}")
    except Exception as e:
        env.add_reply(f"Test call failed: {str(e)}")
        import traceback
        env.add_reply(f"Traceback: {traceback.format_exc()}")

async def agent_response(env: Environment, data_id, response):
    try:
        # Create an account instance with master account credentials
        acc = Account(master_account_id, master_private_key)
        
        # Log debugging info
        env.add_reply(f"Responding as account: {master_account_id}")
        env.add_reply(f"To contract: {contract_id}")
        
        # Convert data_id array to base58 string (standard for NEAR CryptoHash)
        data_id_str = bytes_to_str(data_id)
        env.add_reply(f"data_id as base58: {data_id_str}")
        
        # This format exactly matches what the CLI sends
        json_str = '{"data_id": "' + data_id_str + '", "response": "' + response.replace('"', '\\"').replace('\n', '\\n') + '"}'
        env.add_reply(f"Using CLI-like JSON string: {json_str}")
        
        # Call with the raw JSON string - NO serialization
        tr = await acc.function_call(
            contract_id,
            'agent_response',
            json_str,  # Raw JSON string exactly like CLI
            200000000000000,
            0
        )
        
        env.add_reply(
            f"Transaction created: [{tr.transaction.hash}](https://nearblocks.io/txns/{tr.transaction.hash})")
        return True
    except Exception as e:
        env.add_reply(f"Error in agent_response: {str(e)}")
        import traceback
        env.add_reply(f"Traceback: {traceback.format_exc()}")
        return False

async def simple_test(env: Environment):
    """Simple test that only sends a single string argument"""
    try:
        acc = Account(master_account_id, master_private_key)
        env.add_reply(f"Simple Test - Using account: {master_account_id}")
        
        # Prepare simple args as Python dictionary
        simple_args = {
            "response": "This is a simple test"
        }
        
        env.add_reply(f"Using simple args (Python dict): {simple_args}")
        
        tr = await acc.function_call(
            contract_id,
            'test_agent_response',
            simple_args,  # Pass the Python dictionary directly
            200000000000000,
            0
        )
        env.add_reply(f"Simple test call successful: {tr.transaction.hash}")
    except Exception as e:
        env.add_reply(f"Simple test call failed: {str(e)}")
        import traceback
        env.add_reply(f"Traceback: {traceback.format_exc()}")

async def simplified_agent_response(env: Environment, response):
    """A simplified version that only sends the response string"""
    try:
        acc = Account(master_account_id, master_private_key)
        
        env.add_reply(f"Using simplified agent response with account: {master_account_id}")
        
        # This format exactly matches what the CLI sends
        json_str = '{"response": "' + response.replace('"', '\\"').replace('\n', '\\n') + '"}'
        env.add_reply(f"Using CLI-like JSON string: {json_str}")
        
        # Call with the raw JSON string - NO serialization
        tr = await acc.function_call(
            contract_id,
            'simple_agent_response',
            json_str,  # Raw JSON string exactly like CLI
            200000000000000,
            0
        )
        
        env.add_reply(f"Simplified agent response sent: {tr.transaction.hash}")
        return True
    except Exception as e:
        env.add_reply(f"Simplified agent response failed: {str(e)}")
        import traceback
        env.add_reply(f"Traceback: {traceback.format_exc()}")
        return False

async def public_agent_response(env: Environment, data_id, response):
    """A version that uses the public response endpoint which doesn't check caller identity"""
    try:
        acc = Account(master_account_id, master_private_key)
        
        env.add_reply(f"Using public agent response with account: {master_account_id}")
        
        # Convert data_id array to base58 string (standard for NEAR CryptoHash)
        data_id_str = bytes_to_str(data_id)
        env.add_reply(f"data_id as base58: {data_id_str}")
        
        # This format exactly matches what the CLI sends but using public_agent_response
        json_str = '{"response": "' + response.replace('"', '\\"').replace('\n', '\\n') + '"}'
        env.add_reply(f"Using CLI-like JSON string with public endpoint: {json_str}")
        
        # Call the public endpoint that doesn't check caller identity
        tr = await acc.function_call(
            contract_id,
            'public_agent_response',
            json_str,
            200000000000000,
            0
        )
        
        env.add_reply(
            f"Public agent response sent: [{tr.transaction.hash}](https://nearblocks.io/txns/{tr.transaction.hash})")
        return True
    except Exception as e:
        env.add_reply(f"Public agent response failed: {str(e)}")
        import traceback
        env.add_reply(f"Traceback: {traceback.format_exc()}")
        return False

async def main(env: Environment):
    try:
        message = env.get_last_message()
        env.add_reply(f"Received message: {message}")
        
        # Try to parse as JSON for contract events
        try:
            message_data = json.loads(message["content"])
            # If it's a contract event
            if isinstance(message_data, dict) and "event" in message_data and "data" in message_data:
                event = message_data.get("event")
                event_data = message_data.get("data", [])[0] if message_data.get("data") else {}
                request_id = event_data.get("request_id")
                user_message = event_data.get("message")

                env.add_reply(f"Parsed event: {event}")
                env.add_reply(f"Request ID: {request_id}")
                env.add_reply(f"User message: {user_message}")

                if event == "run_agent" and user_message is not None and request_id is not None:
                    # Process the user's prompt using the LLM
                    prompt = {"role": "system", "content": "You are a helpful AI assistant."}
                    result = env.completion([prompt, {"role": "user", "content": user_message}])
                    
                    env.add_reply(f"Generated AI response: {result}")
                    
                    # Try the public agent response function that doesn't check identity
                    env.add_reply("Using public_agent_response that doesn't check caller identity...")
                    success = await public_agent_response(env, request_id, result)
                    
                    if success:
                        env.add_reply("✅ Public agent response succeeded! Your answer should be delivered.")
                    else:
                        # Try simplified as a fallback
                        env.add_reply("⚠️ Trying simplified_agent_response as fallback...")
                        fallback_success = await simplified_agent_response(env, result)
                        
                        if fallback_success:
                            env.add_reply("✅ Fallback response succeeded.")
                        else:
                            env.add_reply("❌ All response methods failed. Check contract configuration.")
                else:
                    env.add_reply("Invalid request format - missing event, user_message, or request_id")
            else:
                # Handle as direct message
                raise json.JSONDecodeError("Not a contract event", "", 0)
        except (json.JSONDecodeError, KeyError) as e:
            env.add_reply(f"Error parsing message: {str(e)}")
            # Handle as direct message
            prompt = {"role": "system", "content": "You are a helpful AI assistant."}
            result = env.completion([prompt, {"role": "user", "content": message["content"]}])
            env.add_reply(result)

    except Exception as e:
        env.add_reply(f"Error in main: {str(e)}")
        import traceback
        env.add_reply(f"Traceback: {traceback.format_exc()}")

if not (master_account_id and master_private_key):
    env.add_reply("Agent wasn't initialized yet.")
    env.add_reply(f"master_account_id present: {master_account_id is not None}")
    env.add_reply(f"master_private_key present: {master_private_key is not None}")
else:
    asyncio.run(main(env))

env.mark_done()

