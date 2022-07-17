use super::*;

pub struct IxMinAccountsFilter {
    min_accounts: usize,
}

impl IxMinAccountsFilter {
    pub fn new(min_accounts: usize) -> Self {
        Self { min_accounts }
    }
}

impl IxFilter for IxMinAccountsFilter {
    fn filter(&self, ix: &UiCompiledInstruction, _account_keys: Vec<String>) -> bool {
        ix.accounts.len() >= self.min_accounts
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
