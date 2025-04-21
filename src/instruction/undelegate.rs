use crate::{
    error::MyProgramError,
    state::{DataLen, Escrow},
};
use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use bytemuck::{from_bytes, Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_log::log;

pub const DELEGATION_ACCOUNT: Pubkey =
    pinocchio_pubkey::pubkey!("DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh");

pub fn process_undelegate(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, pda_acc, buffer_acc, delegation_record, delegation_metadata, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Get seeds
    let buffer_seeds: &[&[u8]] = &[b"buffer", pda_acc.key().as_ref()];
    let escrow_seeds = &["escrow".as_bytes(), maker.key().as_ref()];
    let delegation_rec_seeds = &[b"delegation", delegation_record.key().as_ref()];
    let delegation_met_seeds = &[b"delegation-metadata", delegation_metadata.key().as_ref()];

    // Find PDAs
    let (_, delegate_account_bump) = pubkey::find_program_address(escrow_seeds, &crate::ID);
    let (_, buffer_pda_bump) = pubkey::find_program_address(buffer_seeds, &crate::ID);

    // Get signer seeds
    let bump = [delegate_account_bump];
    let seed_a = [
        Seed::from(b"escrow"),
        Seed::from(maker.key().as_ref()),
        Seed::from(&bump),
    ];
    let pda_signer_seeds = Signer::from(&seed_a);

    let bump = [buffer_pda_bump];
    let seed_b = [
        Seed::from(b"buffer"),
        Seed::from(pda_acc.key().as_ref()),
        Seed::from(&bump),
    ];
    let buffer_signer_seeds = Signer::from(&seed_b);

    // Close delegated account
    unsafe {
        *maker.borrow_mut_lamports_unchecked() += *pda_acc.borrow_lamports_unchecked();
        *pda_acc.borrow_mut_lamports_unchecked() = 0;
    }
    pda_acc.realloc(0, false)?;
    unsafe { pda_acc.assign(system_program.key()) };

    // Recreate original account with proper owner
    pinocchio_system::instructions::CreateAccount {
        from: maker,
        to: pda_acc,
        lamports: Rent::get()?.minimum_balance(Escrow::LEN),
        space: Escrow::LEN as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&[pda_signer_seeds.clone()])?;

    // Restore data from buffer
    let buffer_data = buffer_acc.try_borrow_data()?;
    let mut pda_data = pda_acc.try_borrow_mut_data()?;
    pda_data.copy_from_slice(&buffer_data);
    drop(pda_data);
    drop(buffer_data);

    // Close buffer account
    unsafe {
        *maker.borrow_mut_lamports_unchecked() += *buffer_acc.borrow_lamports_unchecked();
        *buffer_acc.borrow_mut_lamports_unchecked() = 0;
    }
    buffer_acc.realloc(0, false)?;
    unsafe { buffer_acc.assign(system_program.key()) };

    // Create instruction for delegation program
    let account_metas = vec![
        AccountMeta::new(maker.key(), true, true),
        AccountMeta::new(pda_acc.key(), true, false),
        AccountMeta::new(buffer_acc.key(), false, false),
        AccountMeta::new(delegation_record.key(), true, false),
        AccountMeta::readonly(delegation_metadata.key()),
        AccountMeta::readonly(system_program.key()),
    ];

    let instruction = Instruction {
        program_id: &DELEGATION_ACCOUNT,
        accounts: &account_metas,
        data,
    };

    let acc_infos = [
        maker,
        pda_acc,
        buffer_acc,
        delegation_record,
        delegation_metadata,
        system_program,
    ];

    invoke_signed(&instruction, &acc_infos, &[pda_signer_seeds])?;
    Ok(())
}
