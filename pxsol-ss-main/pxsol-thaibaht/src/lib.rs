#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, system_program,
    sysvar::{self, Sysvar},
};

solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    assert!(data.len() > 0);
    match data[0] {
        0x00 => process_instruction_mint(program_id, accounts, &data[1..]),
        0x01 => process_instruction_transfer(program_id, accounts, &data[1..]),
        _ => unreachable!(),
    }
}

pub fn process_instruction_mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account_user = next_account_info(accounts_iter)?; //用户账户
    let account_user_pda = next_account_info(accounts_iter)?; //数据账户
    let system_program_account = next_account_info(accounts_iter)?; //系统账户
    let rent_sysvar_account = next_account_info(accounts_iter)?; //sysvar rent 账户

    // 账户数据验证
    if !account_user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 账户权限验证
    if !account_user.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }
    if !account_user_pda.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    //系统程序账户验证
    if system_program_account.key != &system_program::ID {
        return Err(ProgramError::IncorrectAuthority);
    }

    //系统租赁账户验证
    if rent_sysvar_account.key != &sysvar::rent::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // PDA 账户按需初始化
    if **account_user_pda.try_borrow_lamports().unwrap() == 0 {
        let rent_exemption = Rent::get()?.minimum_balance(8);
        let bump_seed = Pubkey::find_program_address(&[&account_user.key.to_bytes()], program_id).1;
        invoke_signed(
            &system_instruction::create_account(
                account_user.key,
                account_user_pda.key,
                rent_exemption,
                8,
                program_id,
            ),
            accounts,
            &[&[&account_user.key.to_bytes(), &[bump_seed]]],
        )?;
        account_user_pda
            .data
            .borrow_mut()
            .copy_from_slice(&u64::MIN.to_be_bytes());
    }

    let mut buf = [0u8; 8];
    buf.copy_from_slice(&account_user_pda.data.borrow());
    let old = u64::from_be_bytes(buf);
    buf.copy_from_slice(&data);
    let inc = u64::from_be_bytes(buf);
    let new = old.checked_add(inc).unwrap();
    account_user_pda
        .data
        .borrow_mut()
        .copy_from_slice(&new.to_be_bytes());

    Ok(())
}

pub fn process_instruction_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let account_user = next_account_info(account_iter)?; //用户账户
    let account_user_pda = next_account_info(account_iter)?; //用户pda账户
    let account_into = next_account_info(account_iter)?; // 转入账户
    let account_into_pda = next_account_info(account_iter)?; //转入账户的pda账户
    let _system_program_account = next_account_info(account_iter)?; //系统账户
    let _rent_sysvar_account = next_account_info(account_iter)?; //sysvar rent 账户

    assert!(account_user.is_signer);
    let account_user_pda_calc =
        Pubkey::find_program_address(&[&account_user.key.to_bytes()], program_id);
    assert_eq!(account_user_pda.key, &account_user_pda_calc.0);
    let account_into_pda_calc =
        Pubkey::find_program_address(&[&account_into.key.to_bytes()], program_id);
    assert_eq!(account_into.key, &account_into_pda_calc.0);

    // PDA 账户按需初始化
    if **account_into_pda.try_borrow_lamports().unwrap() == 0 {
        let rent_exmption = Rent::get()?.minimum_balance(8); // 计算 8字节 免租所需的最低 lamports
        let bump_seed = Pubkey::find_program_address(&[&account_into.key.to_bytes()], program_id).1; //计算该 PDA 的正确 bump
        invoke_signed(
            &system_instruction::create_account(
                account_user.key,
                account_into_pda.key,
                rent_exmption,
                8,
                program_id,
            ),
            accounts,
            &[&[&account_into.key.to_bytes(), &[bump_seed]]],
        )?;

        account_into_pda
            .data
            .borrow_mut()
            .copy_from_slice(&u64::MIN.to_be_bytes());
    }

    let mut buf = [0u8; 8];
    buf.copy_from_slice(&account_user_pda.data.borrow());
    let old_user = u64::from_be_bytes(buf);
    buf.copy_from_slice(&account_into_pda.data.borrow());
    let old_into = u64::from_be_bytes(buf);
    buf.copy_from_slice(&data);
    let inc = u64::from_be_bytes(buf);
    let new_user = old_user.checked_sub(inc).unwrap();
    let new_into = old_into.checked_add(inc).unwrap();
    account_user_pda
        .data
        .borrow_mut()
        .copy_from_slice(&new_user.to_be_bytes());
    account_into_pda
        .data
        .borrow_mut()
        .copy_from_slice(&new_into.to_be_bytes());

    Ok(())
}
