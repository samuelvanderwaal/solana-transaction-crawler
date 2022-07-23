use super::*;

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
