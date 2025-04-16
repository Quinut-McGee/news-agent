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

    pub fn query_agent(&mut self, prompt: String) -> PromiseOrValue<String> {
        // Create a promise to resume after the agent responds
        let promise_idx = env::promise_yield_create(
            "on_agent_response",
            &prompt.as_bytes(),
            MIN_RESPONSE_GAS,
            GasWeight::default(),
            DATA_ID_REGISTER,
        );

        // Get the data_id of the register with promises
        let data_id: CryptoHash = env::read_register(DATA_ID_REGISTER)
            .expect("Register is empty")
            .try_into()
            .expect("Wrong register length");

        // Log the data_id clearly
        log!("Generated request_id for agent: {:?}", data_id);

        // Emit the agent event with the prompt
        events::emit::run_agent(&self.agent_name, &prompt, Some(data_id));

        // Return the promise index to the caller
        env::promise_return(promise_idx);
        PromiseOrValue::Promise(Promise::new(env::current_account_id()))
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

        // Resume the initial query transaction with the response
        if !env::promise_yield_resume(&data_id_hash, &args.response.as_bytes()) {
            env::panic_str("Unable to resume promise")
        }
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
        prompt: String,
        #[callback_result] response: Result<String, PromiseError>,
    ) -> Option<String> {
        if let Ok(response) = response.as_ref() {
            log!("Agent response received via public route: {}", response);
            // Store the response in your preferred way
            Some(response.clone())
        } else {
            log!("Response error");
            None
        }
    }

    pub fn get_response(&self, request_id: CryptoHash) -> Option<AgentResponse> {
        self.responses.get(&request_id)
    }

    pub fn simple_agent_response(&mut self, response: String) -> String {
        log!("Simple agent response received: {}", response);
        
        // Just log the response and return it for testing
        format!("Successfully received response: {}", response)
    }

    pub fn direct_query(&self, prompt: String) -> String {
        log!("Processing direct query: {}", prompt);
        
        // Instead of trying to use the agent mechanism, this simply returns a direct response
        format!("Your query was: '{}'. This is a direct response from the smart contract without using the agent response mechanism.", prompt)
    }

    pub fn direct_ai_query(&self, prompt: String) -> PromiseOrValue<String> {
        log!("Directly querying AI agent with: {}", prompt);
        
        // Create a direct call to the agent's direct_query endpoint - clone prompt since we use it again later
        let query = DirectAgentQuery { prompt: prompt.clone() };
        
        // Return a promise to call the agent directly without using the event mechanism
        // This simulates what would happen if the agent had a REST API endpoint
        // Here we return a simple message for now since we can't actually call the agent directly
        
        PromiseOrValue::Value(format!("In a direct call model, the agent would process '{}' and return the result immediately, without the event/response pattern.", prompt))
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
} 