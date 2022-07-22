use super::*;

pub enum IxNumberAccounts {
    LessThan(usize),
    LessThanOrEqual(usize),
    EqualTo(usize),
    GreaterThan(usize),
    GreaterThanOrEqual(usize),
}

impl IxFilter for IxNumberAccounts {
    fn filter(&self, ix: &UiCompiledInstruction, _account_keys: Vec<String>) -> bool {
        match self {
            IxNumberAccounts::LessThan(n) => ix.accounts.len() < *n,
            IxNumberAccounts::LessThanOrEqual(n) => ix.accounts.len() <= *n,
            IxNumberAccounts::EqualTo(n) => ix.accounts.len() == *n,
            IxNumberAccounts::GreaterThan(n) => ix.accounts.len() > *n,
            IxNumberAccounts::GreaterThanOrEqual(n) => ix.accounts.len() >= *n,
        }
    }
}

pub struct IxProgramIdFilter {
    program_id: String,
}

impl IxProgramIdFilter {
    pub fn new(program_id: &str) -> Self {
        Self {
            program_id: program_id.to_string(),
        }
    }
}

impl IxFilter for IxProgramIdFilter {
    fn filter(&self, ix: &UiCompiledInstruction, account_keys: Vec<String>) -> bool {
        account_keys[ix.program_id_index as usize] == self.program_id
    }
}
