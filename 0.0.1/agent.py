import asyncio
import json
import base58
from nearai.agents.environment import Environment
from py_near.account import Account

master_account_id = globals()['env'].env_vars.get("master_account_id", None)
master_private_key = globals()['env'].env_vars.get("master_private_key", None)
contract_id = "dbrd.near"  # Contract ID

async def agent_response(env: Environment, data_id, response):
    """Send a response to the smart contract using direct function call"""
    # Create an account instance with master account credentials
    acc = Account(master_account_id, master_private_key)
    
    # Prepare arguments for the function call
    args = {
        "data_id": data_id,
        "response": response
    }
    
    # Call the smart contract function 'agent_response' with the prepared arguments
    tr = await acc.function_call(contract_id, 'agent_response', args, 200000000000000, 0)
    
    # Add a reply to the agent environment with the transaction hash
    env.add_reply(
        f"Transaction created: [{tr.transaction.hash}](https://nearblocks.io/txns/{tr.transaction.hash})")

async def simple_response(env: Environment, response):
    """Send a simple response with only the response string"""
    # Create an account instance with master account credentials
    acc = Account(master_account_id, master_private_key)
    
    # Prepare a simpler argument structure
    args = {
        "response": response
    }
    
    # Call the smart contract with a simpler method
    tr = await acc.function_call(contract_id, 'public_agent_response', args, 200000000000000, 0)
    
    # Add a reply to the agent environment with the transaction hash
    env.add_reply(
        f"Simple response sent: [{tr.transaction.hash}](https://nearblocks.io/txns/{tr.transaction.hash})")

async def test_set_response(env: Environment, base58_id, response):
    """Set a response with the test_set_response method"""
    # Create an account instance with master account credentials
    acc = Account(master_account_id, master_private_key)
    
    # Prepare arguments
    args = {
        "base58_id": base58_id,
        "response": response
    }
    
    # Call the smart contract
    tr = await acc.function_call(contract_id, 'test_set_response', args, 200000000000000, 0)
    
    # Add a reply to the agent environment with the transaction hash
    env.add_reply(
        f"Test response set: [{tr.transaction.hash}](https://nearblocks.io/txns/{tr.transaction.hash})")

async def main(env: Environment):
    message = env.get_last_message()
    env.add_reply(f"Received message: {message['content']}")

    # Health check handling
    if "health check" in message["content"].lower():
        env.add_reply("Agent health check:")
        env.add_reply(f"✅ master_account_id: {master_account_id}")
        env.add_reply(f"✅ master_private_key: {'*' * 10} (present)")
        env.add_reply(f"✅ contract_id: {contract_id}")
        return

    try:
        # Parse the message as JSON
        message_data = json.loads(message["content"])
        
        # Get basic event information
        event = message_data.get("event")
        
        if event == "run_agent" and len(message_data.get("data", [])) > 0:
            # Extract data from the event
            data = message_data["data"][0]
            request_id = data.get("request_id")
            user_message = data.get("message")
            agent_name = data.get("agent")
            
            env.add_reply(f"Processing request: {user_message}")
            
            # Check if this event is for our agent
            if agent_name == "news-agent" and user_message and request_id:
                # Generate a response using the LLM
                prompt = {"role": "system", "content": "You are a helpful AI assistant."}
                result = env.completion([prompt, {"role": "user", "content": user_message}])
                
                # Convert request_id to base58 string if it's a list
                if isinstance(request_id, list):
                    request_id_str = base58.b58encode(bytes(request_id)).decode('utf-8')
                else:
                    request_id_str = request_id
                
                # Try multiple methods to respond, in order of preference
                try:
                    # First try test_set_response (most reliable)
                    await test_set_response(env, request_id_str, result)
                except Exception as e1:
                    env.add_reply(f"test_set_response failed: {str(e1)}")
                    try:
                        # Next try the standard agent_response
                        await agent_response(env, request_id_str, result)
                    except Exception as e2:
                        env.add_reply(f"agent_response failed: {str(e2)}")
                        try:
                            # Finally try simple_response
                            await simple_response(env, result)
                        except Exception as e3:
                            env.add_reply(f"All response methods failed")
                            env.add_reply(f"Final error: {str(e3)}")
            else:
                env.add_reply("Not for this agent or missing data")
        else:
            # Direct message handling
            prompt = {"role": "system", "content": "You are a helpful AI assistant."}
            result = env.completion([prompt, {"role": "user", "content": message["content"]}])
            env.add_reply(result)
    except Exception as e:
        env.add_reply(f"Error: {str(e)}")
        # Fallback to treating as a direct message
        prompt = {"role": "system", "content": "You are a helpful AI assistant."}
        result = env.completion([prompt, {"role": "user", "content": message["content"]}])
        env.add_reply(result)

# Main entry point
if not (master_account_id and master_private_key):
    env.add_reply("⚠️ Agent wasn't initialized yet.")
    env.add_reply(f"master_account_id present: {master_account_id is not None}")
    env.add_reply(f"master_private_key present: {master_private_key is not None}")
    env.add_reply("Please make sure to set the environment variables in the NEAR AI Hub.")
    env.add_reply("Required variables: master_account_id, master_private_key")
else:
    # Run main function
    asyncio.run(main(env))

env.mark_done()

