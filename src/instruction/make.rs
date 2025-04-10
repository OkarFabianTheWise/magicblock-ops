use bytemuck::{from_bytes, Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_log::log;
use pinocchio_token::state::TokenAccount;

use crate::{
    error::MyProgramError,
    state::{DataLen, Escrow},
};

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct MakeEscrowIx {
    pub bump: u8,
    pub amount_a: [u8; 8],
    pub amount_b: [u8; 8],
}

pub fn process_make(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, mint_a, mint_b, maker_ata, vault, escrow, _system_program, _token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    /*
     if data.len() < core::mem::size_of::<MakeEscrowIx>() {
         return Err(ProgramError::InvalidInstructionData);
     }

     let ix_data: &MakeEscrowIx = from_bytes(data);
    */

    //without bytemuck and raw pointers:
    if data.len() < 17 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let bump = data[0];
    let amount_a = u64::from_le_bytes(
        data[1..9]
            .try_into()
            .map_err(|_| MyProgramError::DeserializationFailed)?,
    );
    let amount_b = data[9..17]
        .try_into()
        .map_err(|_| MyProgramError::DeserializationFailed)?;

    let seeds = &["escrow".as_bytes(), maker.key().as_ref()];

    let (pda, bump_1) =
        pubkey::try_find_program_address(seeds, &crate::ID).ok_or(ProgramError::InvalidSeeds)?;

    log!("bomp {}", bump_1);
    assert_eq!(&pda, escrow.key());

    //is escrow the vault onwer?
    assert!(unsafe {
        TokenAccount::from_account_info_unchecked(vault)
            .map_err(|_| MyProgramError::DeserializationFailed)?
            .owner()
            == escrow.key()
    });

    //has the escrow bee initialized? - check or lamports and data
    if unsafe { escrow.owner() } == &crate::ID {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Create Escrow Account
    pinocchio_system::instructions::CreateAccount {
        from: maker,
        to: escrow,
        lamports: Rent::get()?.minimum_balance(Escrow::LEN),
        space: Escrow::LEN as u64,
        owner: &crate::ID,
    }
    .invoke()?;

    // Populate Escrow Account
    Escrow::initialize(
        escrow,
        *maker.key(),
        *mint_a.key(),
        *mint_b.key(),
        amount_b,
        bump_1,
    );

    pinocchio_token::instructions::Transfer {
        from: maker_ata,
        to: vault,
        authority: maker,
        amount: amount_a,
    }
    .invoke()?;

    Ok(())
}
