use super::*;

pub enum IxNumberAccounts {
    LessThan(usize),
    GreaterThan(usize),
    EqualTo(usize),
}

impl IxFilter for IxNumberAccounts {
    fn filter(&self, ix: &UiParsedInstruction) -> bool {
        match ix {
            UiParsedInstruction::PartiallyDecoded(ix) => match self {
                IxNumberAccounts::LessThan(n) => ix.accounts.len() < *n,
                IxNumberAccounts::GreaterThan(n) => ix.accounts.len() > *n,
                IxNumberAccounts::EqualTo(n) => ix.accounts.len() == *n,
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
