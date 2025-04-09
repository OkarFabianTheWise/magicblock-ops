use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_token::state::TokenAccount;

use crate::state::{DataLen, Escrow};

pub fn process_make(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, mint_a, mint_b, maker_ata, vault, escrow, _system_program, _token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    //bytemuck here?
    let bump = unsafe { *(data.as_ptr() as *const u8) };
    let amount = unsafe { *(data.as_ptr().add(1 + 8) as *const u64) }.to_be_bytes();

    let seed = [(b"escrow"), maker.key().as_slice(), &[bump]];
    let seeds = &seed[..];

    let pda = pubkey::checked_create_program_address(seeds, &crate::ID).unwrap();
    assert_eq!(&pda, escrow.key());

    //is escrow the vault onwer?
    assert!(unsafe {
        TokenAccount::from_account_info_unchecked(vault)
            .unwrap()
            .owner()
            == escrow.key()
    });

    //has the escrow bee initialized?
    if unsafe { escrow.owner() } != &crate::ID {
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
        amount,
        bump,
    );

    pinocchio_token::instructions::Transfer {
        from: maker_ata,
        to: vault,
        authority: maker,
        amount: u64::from_le_bytes(amount),
    }
    .invoke()?;

    Ok(())
}
