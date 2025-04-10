use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::find_program_address,
    ProgramResult,
};

use crate::{
    error::MyProgramError,
    state::{load_acc_mut_unchecked, Escrow},
};

pub fn process_take(accounts: &[AccountInfo]) -> ProgramResult {
    let [taker, maker, mint_a, mint_b, taker_ata_a, taker_ata_b, maker_ata_b, vault, escrow, _token_program, _system_program] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    //here is where we could use bytemuck

    //try to load escrow data:
    let escrow_data = escrow
        .try_borrow_data()
        .map_err(|_| ProgramError::AccountBorrowFailed)?;
    let escrow_account = bytemuck::try_from_bytes::<Escrow>(&escrow_data)
        .map_err(|_| MyProgramError::DeserializationFailed)?;
    ();
    /*
        let escrow_account =
        unsafe { load_acc_mut_unchecked::<Escrow>(escrow.borrow_mut_data_unchecked())? };

    */

    //do we need this? do we need to pass mint_a and mint_b? Isnt the CPI checking this?
    assert_eq!(escrow_account.mint_a, *mint_a.key());
    assert_eq!(escrow_account.mint_b, *mint_b.key());

    let vault_account = pinocchio_token::state::TokenAccount::from_account_info(vault)?;

    let seed = [(b"escrow"), maker.key().as_slice(), &[escrow_account.bump]];
    let seeds = &seed[..];
    let escrow_pda = find_program_address(seeds, &crate::ID).0;
    assert_eq!(*escrow.key(), escrow_pda);

    pinocchio_token::instructions::Transfer {
        from: taker_ata_b,
        to: maker_ata_b,
        authority: taker,
        amount: u64::from_le_bytes(escrow_account.amount),
    }
    .invoke()?;

    let bump = [escrow_account.bump];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.key()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    pinocchio_token::instructions::Transfer {
        from: vault,
        to: taker_ata_a,
        authority: escrow,
        amount: vault_account.amount(),
    }
    .invoke_signed(&[seeds.clone()])?;

    pinocchio_token::instructions::CloseAccount {
        account: vault,
        destination: maker,
        authority: escrow,
    }
    .invoke_signed(&[seeds])?;

    unsafe {
        *maker.borrow_mut_lamports_unchecked() += *escrow.borrow_lamports_unchecked();
        *escrow.borrow_mut_lamports_unchecked() = 0
    };

    Ok(())
}
