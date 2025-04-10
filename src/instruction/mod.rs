pub mod make;
pub mod refund;
pub mod take;

pub use make::*;
pub use refund::*;
pub use take::*;

use pinocchio::program_error::ProgramError;

#[repr(u8)]
pub enum MyProgramInstrution {
    Make,
    Take,
    Refund,
}

impl TryFrom<&u8> for MyProgramInstrution {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(MyProgramInstrution::Make),
            1 => Ok(MyProgramInstrution::Take),
            2 => Ok(MyProgramInstrution::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
