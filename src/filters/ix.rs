use super::*;

/// This filter passes through instructions that match the equality specified by the variant and only
/// applies to PartiallyDecoded instructions. Fully parsed instructions are automatically passed through.
pub enum IxNumberAccounts {
    LessThan(usize),
    LessThanOrEqual(usize),
    EqualTo(usize),
    GreaterThan(usize),
    GreaterThanOrEqual(usize),
}

impl IxFilter for IxNumberAccounts {
    fn filter(&self, ix: &UiParsedInstruction) -> bool {
        match ix {
            UiParsedInstruction::PartiallyDecoded(ix) => match self {
                IxNumberAccounts::LessThan(n) => ix.accounts.len() < *n,
                IxNumberAccounts::LessThanOrEqual(n) => ix.accounts.len() <= *n,
                IxNumberAccounts::EqualTo(n) => ix.accounts.len() == *n,
                IxNumberAccounts::GreaterThan(n) => ix.accounts.len() > *n,
                IxNumberAccounts::GreaterThanOrEqual(n) => ix.accounts.len() >= *n,
            },
            // This filter does not apply to parsed accounts.
            UiParsedInstruction::Parsed(_ix) => true,
        }
    }
}

/// This filter checks that the instruction has the specified program id.
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
    fn filter(&self, ix: &UiParsedInstruction) -> bool {
        match ix {
            UiParsedInstruction::Parsed(ix) => ix.program_id == self.program_id,
            UiParsedInstruction::PartiallyDecoded(ix) => ix.program_id == self.program_id,
        }
    }
}

/// This filter passes through instructions that match the Base58 encoded data for an instruction.
pub struct IxDataFilter {
    data: String,
}

impl IxDataFilter {
    pub fn new(data: &str) -> Self {
        Self {
            data: data.to_string(),
        }
    }
}

impl IxFilter for IxDataFilter {
    fn filter(&self, ix: &UiParsedInstruction) -> bool {
        match ix {
            UiParsedInstruction::PartiallyDecoded(ix) => ix.data == self.data,
            // This filter does not apply to parsed accounts.
            UiParsedInstruction::Parsed(_ix) => false,
        }
    }
}

/// This filter only applies to fully parsed instructions, and passes through any instruction with the type "mintTo".
/// This filter is useful for getting the mintTo instruction from SPL token calls.
pub struct IxMintToFilter;

impl IxFilter for IxMintToFilter {
    fn filter(&self, ix: &UiParsedInstruction) -> bool {
        match ix {
            UiParsedInstruction::Parsed(ix) => ix
                .parsed
                .get("type")
                .map(|type_| type_ == "mintTo")
                .unwrap_or(false),
            // This filter only applies to fully parsed instructions.
            UiParsedInstruction::PartiallyDecoded(_ix) => false,
        }
    }
}

pub struct IxHasAccountFilter {
    account: String,
}

impl IxHasAccountFilter {
    pub fn new(account: &str) -> Self {
        Self {
            account: account.to_string(),
        }
    }
}

impl IxFilter for IxHasAccountFilter {
    fn filter(&self, ix: &UiParsedInstruction) -> bool {
        match ix {
            UiParsedInstruction::Parsed(_ix) => true,
            UiParsedInstruction::PartiallyDecoded(ix) => {
                ix.accounts.iter().any(|account| account == &self.account)
            }
        }
    }
}

pub struct IxHasAccountAtIndexFilter {
    account: String,
    index: usize,
}

impl IxHasAccountAtIndexFilter {
    pub fn new(account: &str, index: usize) -> Self {
        Self {
            account: account.to_string(),
            index,
        }
    }
}

impl IxFilter for IxHasAccountAtIndexFilter {
    fn filter(&self, ix: &UiParsedInstruction) -> bool {
        match ix {
            UiParsedInstruction::Parsed(_ix) => false,
            UiParsedInstruction::PartiallyDecoded(ix) => ix
                .accounts
                .get(self.index)
                .map(|account| account == &self.account)
                .unwrap_or(false),
        }
    }
}
