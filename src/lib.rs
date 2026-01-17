#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, contracterror, Address, Env, Vec};

// #![no_std] means the contract does not use Rust’s standard library, which is required for Soroban.
// soroban_sdk provides types and macros needed for writing a smart contract, accessing storage, handling authentication, and working with addresses and vectors.




#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct BudgetState {
    pub current: i128,
    pub min: i128,
    pub max: i128,
}
// BudgetState stores the budget data:
// current is the current budget value
// min is the lower limit
// max is the upper limit
// This struct represents the main financial state stored on-chain.





#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Owner,
    Operators,
    Budget,
}
// DataKey defines keys used for persistent storage:
// Owner stores the owner address
// Operators stores the list of operators
// Budget stores the BudgetState





#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BudgetError {
    NotOwner = 1,
    NotOperator = 2,
    AlreadyOperator = 3,
    NotOperatorFound = 4,
    Overflow = 5,
    Underflow = 6,
    ExceedsMax = 7,
    BelowMin = 8,
    InvalidLimits = 9,
}

// BudgetError defines all failure cases:
// Access control errors
// Arithmetic errors
// Budget limit violations
// Invalid initialization
// Each error has a numeric value so failures are deterministic and testable.



#[contract]
pub struct GovernanceBudgetAllocator;
// This declares the contract type.
// All callable contract functions are implemented for this struct.



#[contractimpl]
impl GovernanceBudgetAllocator {
    /// Initialize the contract with owner, initial budget, and limits
    pub fn initialize(env: Env, owner: Address, initial: i128, min: i128, max: i128) {
        // Validate limits: min <= initial <= max
        if min > initial || initial > max {
            panic!("Invalid limits: min must be <= initial <= max");
        }
        
        // Store owner
        env.storage().persistent().set(&DataKey::Owner, &owner);
        
        // Initialize empty operators list
        let operators: Vec<Address> = Vec::new(&env);
        env.storage().persistent().set(&DataKey::Operators, &operators);
        
        // Store budget state
        let budget = BudgetState {
            current: initial,
            min,
            max,
        };
        env.storage().persistent().set(&DataKey::Budget, &budget);
    }
//     Creates the initial budget state and stores it.
//      The contract is now fully initialized.
    



//     This function adds a new operator.
//.     The caller must authenticate.
    pub fn add_operator(env: Env, caller: Address, operator: Address) -> Result<(), BudgetError> {
        caller.require_auth();
        
        // Verify caller is owner
//       Checks that the caller is the owner.
//       If not, returns a NotOwner error.
        let owner: Address = env.storage().persistent().get(&DataKey::Owner).unwrap();
        if caller != owner {
            return Err(BudgetError::NotOwner);
        }
        
        // Get operators list
        // Loads the current list of operators from storage.
        let mut operators: Vec<Address> = env.storage().persistent().get(&DataKey::Operators).unwrap();
        
        
        // Checks whether the address is already an operator.
        // Prevents duplicate entries.
        for op in operators.iter() {
            if op == operator {
                return Err(BudgetError::AlreadyOperator);
            }
        }
        
        // Add operator
        operators.push_back(operator);
        env.storage().persistent().set(&DataKey::Operators, &operators);
        
        Ok(())
    }
    
//     Removes an operator.
//    The caller must authenticate.
    pub fn remove_operator(env: Env, caller: Address, operator: Address) -> Result<(), BudgetError> {
        caller.require_auth();
        
        // Verify caller is owner
        let owner: Address = env.storage().persistent().get(&DataKey::Owner).unwrap();
        if caller != owner {
            return Err(BudgetError::NotOwner);
        }
        
        // Get operators list
        let operators: Vec<Address> = env.storage().persistent().get(&DataKey::Operators).unwrap();
        
        // Find and remove operator
        
        let mut found = false;
        let mut new_operators = Vec::new(&env);
        for op in operators.iter() {
            if op == operator {
                found = true;
            } else {
                new_operators.push_back(op);
            }
        }
        
        if !found {
            return Err(BudgetError::NotOperatorFound);
        }
        
        env.storage().persistent().set(&DataKey::Operators, &new_operators);
        
        Ok(())
    }

    // Increase the budget (operators only)
    
    pub fn increase_budget(env: Env, caller: Address, amount: i128) -> Result<i128, BudgetError> {
        caller.require_auth();
        
        // Check if caller is operator
        let operators: Vec<Address> = env.storage().persistent().get(&DataKey::Operators).unwrap();
        let mut is_operator = false;
        for op in operators.iter() {
            if op == caller {
                is_operator = true;
                break;
            }
        }
        
        if !is_operator {
            return Err(BudgetError::NotOperator);
        }
        
        // Get current budget
        let mut budget: BudgetState = env.storage().persistent().get(&DataKey::Budget).unwrap();
        
        // Safe addition with overflow check
        let new_value = budget.current.checked_add(amount)
            .ok_or(BudgetError::Overflow)?;
        
        // Check max limit
        if new_value > budget.max {
            return Err(BudgetError::ExceedsMax);
        }
        
        // Update state
        budget.current = new_value;
        env.storage().persistent().set(&DataKey::Budget, &budget);
        
        Ok(new_value)
    }
    
    // Decrease the budget (operators only)
   
    pub fn decrease_budget(env: Env, caller: Address, amount: i128) -> Result<i128, BudgetError> {
        caller.require_auth();
        
        // Check if caller is operator
        let operators: Vec<Address> = env.storage().persistent().get(&DataKey::Operators).unwrap();
        let mut is_operator = false;
        for op in operators.iter() {
            if op == caller {
                is_operator = true;
                break;
            }
        }
        
        if !is_operator {
            return Err(BudgetError::NotOperator);
        }
        
        // Get current budget
        let mut budget: BudgetState = env.storage().persistent().get(&DataKey::Budget).unwrap();
        
        // Safe subtraction with underflow check
        let new_value = budget.current.checked_sub(amount)
            .ok_or(BudgetError::Underflow)?;
        
        // Check min limit
        if new_value < budget.min {
            return Err(BudgetError::BelowMin);
        }
        
        // Update state
        budget.current = new_value;
        env.storage().persistent().set(&DataKey::Budget, &budget);
        
        Ok(new_value)
    }
    
    
    // Get current budget state
    pub fn get_budget(env: Env) -> BudgetState {
        env.storage().persistent().get(&DataKey::Budget).unwrap()
    }
    
    // Get contract owner address
    pub fn get_owner(env: Env) -> Address {
        env.storage().persistent().get(&DataKey::Owner).unwrap()
    }
    
    // Get list of authorized operators
    pub fn get_operators(env: Env) -> Vec<Address> {
        env.storage().persistent().get(&DataKey::Operators).unwrap()
    }
    
    // Check if an address is an operator
    pub fn is_operator(env: Env, address: Address) -> bool {
        let operators: Vec<Address> = env.storage().persistent().get(&DataKey::Operators).unwrap();
        for op in operators.iter() {
            if op == address {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
// This module contains unit tests for the contract.
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, GovernanceBudgetAllocator);
        let client = GovernanceBudgetAllocatorClient::new(&env, &contract_id);
        
        let owner = Address::generate(&env);
        
        client.initialize(&owner, &1000, &0, &10000);
        
        let budget = client.get_budget();
        assert_eq!(budget.current, 1000);
        assert_eq!(budget.min, 0);
        assert_eq!(budget.max, 10000);
    }
    
    #[test]
    fn test_add_operator() {
        let env = Env::default();
        let contract_id = env.register_contract(None, GovernanceBudgetAllocator);
        let client = GovernanceBudgetAllocatorClient::new(&env, &contract_id);
        
        let owner = Address::generate(&env);
        let operator = Address::generate(&env);
        
        client.initialize(&owner, &1000, &0, &10000);
        
        env.mock_all_auths();
        client.add_operator(&owner, &operator);
        
        assert!(client.is_operator(&operator));
    }
    
    #[test]
    fn test_increase_budget() {
        let env = Env::default();
        let contract_id = env.register_contract(None, GovernanceBudgetAllocator);
        let client = GovernanceBudgetAllocatorClient::new(&env, &contract_id);
        
        let owner = Address::generate(&env);
        let operator = Address::generate(&env);
        
        client.initialize(&owner, &1000, &0, &10000);
        
        env.mock_all_auths();
        client.add_operator(&owner, &operator);
        
        let new_value = client.increase_budget(&operator, &500);
        assert_eq!(new_value, 1500);
    }
    
    #[test]
    #[should_panic(expected = "Error(Contract, #2)")]
    fn test_unauthorized_increase() {
        let env = Env::default();
        let contract_id = env.register_contract(None, GovernanceBudgetAllocator);
        let client = GovernanceBudgetAllocatorClient::new(&env, &contract_id);
        
        let owner = Address::generate(&env);
        let unauthorized = Address::generate(&env);
        
        client.initialize(&owner, &1000, &0, &10000);
        
        env.mock_all_auths();
        client.increase_budget(&unauthorized, &500);
    }
    
    #[test]
    #[should_panic(expected = "Error(Contract, #7)")]
    fn test_exceeds_max_limit() {
        let env = Env::default();
        let contract_id = env.register_contract(None, GovernanceBudgetAllocator);
        let client = GovernanceBudgetAllocatorClient::new(&env, &contract_id);
        
        let owner = Address::generate(&env);
        let operator = Address::generate(&env);
        
        client.initialize(&owner, &1000, &0, &10000);
        
        env.mock_all_auths();
        client.add_operator(&owner, &operator);
        client.increase_budget(&operator, &10000);
    }
}
// Initialization stores correct values
// Owner can add operators
// Operators can increase the budget
// Unauthorized users are blocked
// Budget limits are enforced