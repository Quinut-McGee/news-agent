use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, BorshStorageKey, Gas, PanicOnDefault,
    PromiseOrValue, Promise, PromiseIndex,
};
use near_sdk::{GasWeight, PromiseError};
use serde_json::json;
use std::convert::TryInto;
use schemars::JsonSchema;
use base64;
use bs58;

mod events;

type CryptoHash = [u8; 32];
const TGAS: u64 = 1_000_000_000_000;
pub const MIN_RESPONSE_GAS: Gas = Gas::from_tgas(30);
pub const DATA_ID_REGISTER: u64 = 37;

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    AgentResponses,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct AgentResponse {
    #[schemars(with = "String")]
    pub request_id: CryptoHash,
    pub response: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct AgentResponseArgs {
    pub data_id: String,  // Base58 encoded string of the CryptoHash
    pub response: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct DirectAgentQuery {
    pub prompt: String
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct DirectAgentResponse {
    pub response: String
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct QueryPrompt {
    pub prompt: String
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    agent_account_id: AccountId,
    agent_name: String,
    responses: near_sdk::collections::UnorderedMap<CryptoHash, AgentResponse>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(agent_account_id: AccountId, agent_name: String) -> Self {
        Self {
            agent_account_id,
            agent_name,
            responses: near_sdk::collections::UnorderedMap::new(StorageKey::AgentResponses),
        }
    }

    pub fn query_agent(&mut self, prompt: String) -> String {
        log!("Received query with prompt: {}", prompt);
        
        // Generate a unique ID for this request
        let mut rng_seed = env::random_seed();
        let mut data_id: CryptoHash = [0; 32];
        for i in 0..32 {
            data_id[i] = rng_seed[i % rng_seed.len()];
        }

        // Convert data_id to base58 for easier use by agents
        let data_id_base58 = bs58::encode(&data_id).into_string();
        
        // Log the data_id clearly
        log!("Generated request_id for agent: {:?}", data_id);
        log!("Request ID as base58: {}", data_id_base58);

        // Emit the agent event with the prompt
        events::emit::run_agent(&self.agent_name, &prompt, Some(data_id));
        
        // Store an empty response to indicate the request is pending
        self.responses.insert(
            &data_id,
            &AgentResponse {
                request_id: data_id,
                response: "PENDING".to_string(),
            },
        );

        // Return the base58 request ID that the caller can use to poll for results
        format!("{{\"request_id\": \"{}\", \"status\": \"pending\"}}", data_id_base58)
    }

    pub fn agent_response(&mut self, args: AgentResponseArgs) {
        log!("Agent responded with: {}", args.response);

        assert_eq!(
            env::predecessor_account_id(),
            self.agent_account_id,
            "Illegal agent account_id"
        );

        // Convert base58 string to bytes
        log!("Decoding data_id from base58: {}", args.data_id);
        
        let data_id_bytes = bs58::decode(&args.data_id)
            .into_vec()
            .unwrap_or_else(|e| env::panic_str(&format!("Failed to decode base58: {}", e)));
        
        let data_id_bytes_len = data_id_bytes.len();
        
        // Convert bytes to CryptoHash
        let data_id_hash: CryptoHash = data_id_bytes
            .try_into()
            .unwrap_or_else(|_| env::panic_str(&format!(
                "Invalid data_id length: got {} bytes, expected 32", 
                data_id_bytes_len
            )));

        // Store the response
        self.responses.insert(
            &data_id_hash,
            &AgentResponse {
                request_id: data_id_hash,
                response: args.response.clone(),
            },
        );

        // Try to resume the promise if it exists
        // This is now optional - will work with both old and new approaches
        let _ = env::promise_yield_resume(&data_id_hash, &args.response.as_bytes());
        
        // Emit an event for the response
        log!("Response stored successfully for request_id: {}", args.data_id);
    }

    // Add a simplified test function that only requires the response string
    pub fn test_agent_response(&mut self, response: String) -> String {
        log!("Test agent response received: {}", response);
        
        assert_eq!(
            env::predecessor_account_id(),
            self.agent_account_id,
            "Illegal agent account_id"
        );
        
        format!("Test successful, received: {}", response)
    }

    #[private]
    pub fn on_agent_response(
        &mut self,
        #[callback_unwrap] prompt: String,
        #[callback_result] response: Result<String, PromiseError>,
    ) -> Option<String> {
        if let Ok(response) = response.as_ref() {
            log!("Agent response received via yield resume: {}", response);
            
            // Generate a unique ID for this response
            let mut rng_seed = env::random_seed();
            let mut data_id: CryptoHash = [0; 32];
            for i in 0..32 {
                data_id[i] = rng_seed[i % rng_seed.len()];
            }
            
            // Store the response for future retrieval
            self.responses.insert(
                &data_id,
                &AgentResponse {
                    request_id: data_id,
                    response: response.clone(),
                },
            );
            
            Some(response.clone())
        } else {
            log!("Response error or timeout");
            None
        }
    }

    pub fn get_response(&self, request_id: CryptoHash) -> Option<AgentResponse> {
        self.responses.get(&request_id)
    }

    pub fn get_response_by_id(&self, base58_id: String) -> Option<String> {
        log!("Looking up response for ID: {}", base58_id);
        
        // Convert base58 ID to CryptoHash
        let id_bytes = match bs58::decode(&base58_id).into_vec() {
            Ok(bytes) => bytes,
            Err(e) => {
                log!("Failed to decode base58 ID: {}", e);
                return None;
            }
        };
        
        if id_bytes.len() != 32 {
            log!("Invalid ID length: {} (expected 32)", id_bytes.len());
            return None;
        }
        
        let crypto_hash: CryptoHash = match id_bytes.try_into() {
            Ok(hash) => hash,
            Err(_) => {
                log!("Failed to convert bytes to CryptoHash");
                return None;
            }
        };
        
        // Get the response
        match self.responses.get(&crypto_hash) {
            Some(response) => {
                log!("Found response for ID: {}", base58_id);
                Some(response.response)
            },
            None => {
                log!("No response found for ID: {}", base58_id);
                None
            }
        }
    }

    pub fn simple_agent_response(&mut self, response: String) -> String {
        log!("Simple agent response received: {}", response);
        
        // Just log the response and return it for testing
        format!("Successfully received response: {}", response)
    }

    pub fn simple_agent_query(&mut self, prompt: String) -> String {
        // Generate a unique ID for this request
        let mut rng_seed = env::random_seed();
        let mut data_id: CryptoHash = [0; 32];
        for i in 0..32 {
            data_id[i] = rng_seed[i % rng_seed.len()];
        }

        // Log the data_id clearly
        log!("Generated request_id for simple query: {:?}", data_id);

        // Emit the agent event with the prompt
        events::emit::run_agent(&self.agent_name, &prompt, Some(data_id));

        format!("Agent query sent with request_id: {:?}. The agent will process this request but you won't receive the response in this transaction.", data_id)
    }

    pub fn direct_query(&mut self, prompt: String) -> String {
        log!("Processing direct query: {}", prompt);
        
        // Generate a unique ID for this request
        let mut rng_seed = env::random_seed();
        let mut data_id: CryptoHash = [0; 32];
        for i in 0..32 {
            data_id[i] = rng_seed[i % rng_seed.len()];
        }

        // Log the data_id clearly
        log!("Generated request_id for direct query: {:?}", data_id);

        // Emit the agent event with the prompt
        events::emit::run_agent(&self.agent_name, &prompt, Some(data_id));
        
        // Instead of trying to use the agent mechanism, this simply returns a direct response
        format!("Agent event emitted for prompt: '{}'. The agent should now process this, but the response will not be returned in this transaction.", prompt)
    }

    pub fn direct_ai_query(&mut self, prompt: String) -> String {
        log!("Directly querying AI agent with: {}", prompt);
        
        // Generate a unique ID for this request
        let mut rng_seed = env::random_seed();
        let mut data_id: CryptoHash = [0; 32];
        for i in 0..32 {
            data_id[i] = rng_seed[i % rng_seed.len()];
        }

        // Log the data_id clearly
        log!("Generated request_id for AI query: {:?}", data_id);

        // Emit the agent event with the prompt
        events::emit::run_agent(&self.agent_name, &prompt, Some(data_id));
        
        format!("Agent event emitted for prompt: '{}'. The agent should now process this request. Check your agent logs for the response.", prompt)
    }

    pub fn public_agent_response(&mut self, response: String) -> String {
        log!("Public agent response received: {}", response);
        
        // This function doesn't check the predecessor account ID,
        // so it can be called by anyone
        
        format!("Response received and processed: {}", response)
    }

    pub fn direct_llm_query(&self, prompt: String) -> String {
        log!("Processing direct LLM query: {}", prompt);

        // Instead of trying to emit an event for the agent to process, we would
        // ideally call an API directly here. Since we can't do that from the contract,
        // we're returning a simulated response.
        
        let simulated_responses = [
            "Yes, sharks are amazing creatures that have evolved over millions of years into perfectly adapted marine predators.",
            "The greatness of any animal is subjective, but sharks are certainly among the most impressive creatures in the ocean.",
            "Sharks play a vital role in marine ecosystems as apex predators, helping maintain the balance of ocean life.",
            "While sharks are incredible, every animal has its unique qualities that make it 'great' in its own way.",
            "Sharks have survived multiple mass extinctions and have been on Earth for over 450 million years, which is quite impressive!"
        ];
        
        // Select a response based on the hash of the prompt
        let response_index = prompt.bytes().fold(0, |sum, b| sum + b as usize) % simulated_responses.len();
        
        format!("Direct LLM Response: {}", simulated_responses[response_index])
    }

    pub fn respond(&mut self, yield_id: String, response: String) -> String {
        log!("Responding to yield_id: {}", yield_id);
        
        // Decode the yield_id from base58 to CryptoHash bytes
        let yield_id_bytes = bs58::decode(&yield_id)
            .into_vec()
            .unwrap_or_else(|e| env::panic_str(&format!("Failed to decode base58: {}", e)));
        
        let yield_id_bytes_len = yield_id_bytes.len();
        
        // Convert bytes to CryptoHash
        let yield_id_hash: CryptoHash = yield_id_bytes
            .try_into()
            .unwrap_or_else(|_| env::panic_str(&format!(
                "Invalid yield_id length: got {} bytes, expected 32", 
                yield_id_bytes_len
            )));

        // Store the response
        self.responses.insert(
            &yield_id_hash,
            &AgentResponse {
                request_id: yield_id_hash,
                response: response.clone(),
            },
        );

        log!("Stored response for yield_id: {}", yield_id);
        log!("Now attempting to resume the promise...");

        // Resume the yielded promise
        if env::promise_yield_resume(&yield_id_hash, &response.as_bytes()) {
            log!("Promise resumed successfully");
            "Successfully resumed promise".to_string()
        } else {
            log!("Failed to resume promise");
            env::panic_str("Failed to resume promise")
        }
    }

    pub fn query_agent_async(&mut self, prompt: String) -> String {
        log!("Sending asynchronous query with prompt: {}", prompt);
        
        // Generate a unique ID for this request
        let mut rng_seed = env::random_seed();
        let mut data_id: CryptoHash = [0; 32];
        for i in 0..32 {
            data_id[i] = rng_seed[i % rng_seed.len()];
        }

        // Convert data_id to base58 for easier use by agents and frontend
        let data_id_base58 = bs58::encode(&data_id).into_string();
        
        // Log the data_id
        log!("Generated request_id for async query: {:?}", data_id);
        log!("Request ID as base58: {}", data_id_base58);

        // Emit the agent event with the prompt
        events::emit::run_agent(&self.agent_name, &prompt, Some(data_id));
        
        // Store an empty response to indicate the request is pending
        self.responses.insert(
            &data_id,
            &AgentResponse {
                request_id: data_id,
                response: "PENDING".to_string(),
            },
        );

        // Return the base58 request ID that the frontend can use to poll for results
        format!("{{\"request_id\": \"{}\", \"status\": \"pending\"}}", data_id_base58)
    }

    pub fn test_set_response(&mut self, base58_id: String, response: String) -> String {
        log!("Setting test response for ID: {}", base58_id);
        
        // Convert base58 ID to CryptoHash
        let id_bytes = match bs58::decode(&base58_id).into_vec() {
            Ok(bytes) => bytes,
            Err(e) => {
                log!("Failed to decode base58 ID: {}", e);
                return format!("Error: Failed to decode base58 ID: {}", e);
            }
        };
        
        if id_bytes.len() != 32 {
            log!("Invalid ID length: {} (expected 32)", id_bytes.len());
            return format!("Error: Invalid ID length: {} (expected 32)", id_bytes.len());
        }
        
        let crypto_hash: CryptoHash = match id_bytes.try_into() {
            Ok(hash) => hash,
            Err(_) => {
                log!("Failed to convert bytes to CryptoHash");
                return "Error: Failed to convert bytes to CryptoHash".to_string();
            }
        };
        
        // Store the response
        self.responses.insert(
            &crypto_hash,
            &AgentResponse {
                request_id: crypto_hash,
                response: response.clone(),
            },
        );
        
        log!("Test response set for ID: {}", base58_id);
        format!("Test response set for ID: {}", base58_id)
    }

    pub fn query_agent_direct(&mut self, prompt: String) -> String {
        log!("Processing direct query with async agent: {}", prompt);
        
        // Generate a unique ID for this request
        let mut rng_seed = env::random_seed();
        let mut data_id: CryptoHash = [0; 32];
        for i in 0..32 {
            data_id[i] = rng_seed[i % rng_seed.len()];
        }

        // Convert data_id to base58 for easier use by agents
        let data_id_base58 = bs58::encode(&data_id).into_string();
        
        // Log the data_id
        log!("Generated request_id for direct query: {:?}", data_id);
        log!("Request ID as base58: {}", data_id_base58);

        // Emit the agent event with the prompt
        events::emit::run_agent(&self.agent_name, &prompt, Some(data_id));
        
        // Store an empty response placeholder
        self.responses.insert(
            &data_id,
            &AgentResponse {
                request_id: data_id,
                response: "PENDING".to_string(),
            },
        );

        // Return the base58 request ID that the frontend can use to poll for results
        format!("{{\"request_id\": \"{}\", \"status\": \"pending\"}}", data_id_base58)
    }
} 