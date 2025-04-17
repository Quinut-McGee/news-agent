import asyncio
import json
import base58
from nearai.agents.environment import Environment
from py_near.account import Account

master_account_id = globals()['env'].env_vars.get("master_account_id", None)
master_private_key = globals()['env'].env_vars.get("master_private_key", None)
contract_id = "dbrd.near"  # Contract ID

# Direct response mode (fallback to on-contract response generation)
async def direct_llm_query(env: Environment, prompt):
    """Call direct_llm_query as a fallback method"""
    acc = Account(master_account_id, master_private_key)
    
    args = {
        "prompt": prompt
    }
    
    env.add_reply(f"Calling direct_llm_query with prompt: {prompt}")
    
    tr = await acc.function_call(contract_id, 'direct_llm_query', args, 300000000000000, 0)
    
    env.add_reply(
        f"Direct response generated: [{tr.transaction.hash}](https://nearblocks.io/txns/{tr.transaction.hash})")

async def test_set_response(env: Environment, request_id_str, response):
    """Store a response using test_set_response method"""
    env.add_reply(f"Storing response using test_set_response for ID: {request_id_str}")
    
    # Create an account instance with master account credentials
    acc = Account(master_account_id, master_private_key)
    
    # Prepare arguments
    args = {
        "base58_id": request_id_str,
        "response": response
    }
    
    # Call the smart contract
    tr = await acc.function_call(contract_id, 'test_set_response', args, 300000000000000, 0)
    
    # Add a reply to the agent environment with the transaction hash
    env.add_reply(
        f"Response stored with test_set_response: [{tr.transaction.hash}](https://nearblocks.io/txns/{tr.transaction.hash})")
    return True

async def agent_response(env: Environment, request_id_str, response):
    """Send a response to the smart contract using agent_response method"""
    env.add_reply(f"Storing response using agent_response for ID: {request_id_str}")
    
    # Create an account instance with master account credentials
    acc = Account(master_account_id, master_private_key)
    
    # Prepare arguments for the function call
    args = {
        "data_id": request_id_str,
        "response": response
    }
    
    # Call the smart contract function
    tr = await acc.function_call(contract_id, 'agent_response', args, 300000000000000, 0)
    
    # Add a reply to the agent environment with the transaction hash
    env.add_reply(
        f"Response stored with agent_response: [{tr.transaction.hash}](https://nearblocks.io/txns/{tr.transaction.hash})")
    return True

async def process_event(env: Environment, message_data):
    """Process an agent event with proper debugging"""
    try:
        # Extract data from the event
        event_data = message_data.get("data", [])[0]
        request_id = event_data.get("request_id")
        user_message = event_data.get("message")
        agent_name = event_data.get("agent")
        
        env.add_reply(f"Processing event for agent: {agent_name}")
        env.add_reply(f"User message: {user_message}")
        
        # Only process if it's for our agent
        if agent_name == "news-agent" and user_message and request_id:
            # Generate a response using the LLM
            prompt = {"role": "system", "content": "You are a helpful AI assistant."}
            result = env.completion([prompt, {"role": "user", "content": user_message}])
            env.add_reply(f"Generated response: {result[:100]}...")
            
            # Convert request_id to base58 string if it's a list
            if isinstance(request_id, list):
                request_id_str = base58.b58encode(bytes(request_id)).decode('utf-8')
                env.add_reply(f"Converted request ID to base58: {request_id_str}")
            else:
                request_id_str = request_id
                env.add_reply(f"Using request ID as-is: {request_id_str}")
            
            # Try to store the response using test_set_response first
            try:
                await test_set_response(env, request_id_str, result)
                env.add_reply("Successfully stored response!")
                return
            except Exception as e1:
                env.add_reply(f"Failed to store response with test_set_response: {str(e1)}")
            
            # Try agent_response as fallback
            try:
                await agent_response(env, request_id_str, result)
                env.add_reply("Successfully stored response with agent_response!")
                return
            except Exception as e2:
                env.add_reply(f"Failed to store response with agent_response: {str(e2)}")
            
            # Final fallback - try direct_llm_query
            try:
                await direct_llm_query(env, user_message)
                env.add_reply("Successfully generated a direct response!")
            except Exception as e3:
                env.add_reply(f"All response methods failed!")
        else:
            env.add_reply(f"Event not for this agent or missing data. Agent name: {agent_name}")
    except Exception as e:
        env.add_reply(f"Error processing event: {str(e)}")
        import traceback
        env.add_reply(f"Traceback: {traceback.format_exc()}")

async def main(env: Environment):
    try:
        # Get the last message
        message = env.get_last_message()
        env.add_reply(f"Received message of type: {type(message['content'])}")
        
        # Special command handling
        if "health check" in message["content"].lower():
            env.add_reply("Agent health check:")
            env.add_reply(f"✅ master_account_id: {master_account_id}")
            env.add_reply(f"✅ master_private_key: {'*' * 10} (present)")
            env.add_reply(f"✅ contract_id: {contract_id}")
            return
        
        if "test:" in message["content"].lower():
            # Test mode for direct responses
            test_prompt = message["content"].split("test:", 1)[1].strip()
            await direct_llm_query(env, test_prompt)
            return
        
        # Try to parse as JSON (event)
        try:
            message_data = json.loads(message["content"])
            env.add_reply(f"Received JSON message with keys: {list(message_data.keys())}")
            
            # Check for run_agent event
            if message_data.get("event") == "run_agent" and "data" in message_data:
                env.add_reply("Received run_agent event")
                await process_event(env, message_data)
            else:
                env.add_reply("Not a valid run_agent event format")
                # Process as normal message
                prompt = {"role": "system", "content": "You are a helpful AI assistant."}
                result = env.completion([prompt, {"role": "user", "content": message["content"]}])
                env.add_reply(result)
        except json.JSONDecodeError:
            # Handle as normal message
            env.add_reply("Received non-JSON message, treating as direct query")
            prompt = {"role": "system", "content": "You are a helpful AI assistant."}
            result = env.completion([prompt, {"role": "user", "content": message["content"]}])
            env.add_reply(result)
    except Exception as e:
        env.add_reply(f"Error in main: {str(e)}")
        import traceback
        env.add_reply(f"Traceback: {traceback.format_exc()}")

# Main entry point
if not master_account_id or not master_private_key:
    env.add_reply("⚠️ Agent wasn't initialized yet.")
    env.add_reply(f"master_account_id present: {master_account_id is not None}")
    env.add_reply(f"master_private_key present: {master_private_key is not None}")
    env.add_reply("Please make sure to set the environment variables in the NEAR AI Hub.")
else:
    # Run main function for all messages
    asyncio.run(main(env))

env.mark_done()

