#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, system_program,
    sysvar::{self, Sysvar},
};

//最大数据大小 10 KB
const MAX_DATA_SIZE: usize = 10 * 1024;

solana_program::entrypoint!(process_instruction);

// 程序入口函数
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    solana_program::msg!("Hello Solana");

    if data.len() > MAX_DATA_SIZE {
        return Err(ProgramError::InvalidAccountData);
    }

    // 获取涉及到的账户信息
    let accounts_iter = &mut accounts.iter();
    // 用户账户
    let account_user = next_account_info(accounts_iter)?;
    // 数据账户
    let account_data = next_account_info(accounts_iter)?;
    // 系统账户
    let system_program_account = next_account_info(accounts_iter)?;
    // sysvar rent 账户
    let rent_sysvar_account = next_account_info(accounts_iter)?;

    // 账户数据校验
    if !account_user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 验证账户权限
    if !account_user.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }
    if !account_data.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    // 验证系统程序帐号
    if system_program_account.key != &system_program::ID {
        return Err(ProgramError::IncorrectAuthority);
    }

    // 验证租用系统帐户
    if rent_sysvar_account.key != &sysvar::rent::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    //计算租赁豁免
    let rent_exemption = Rent::get()?.minimum_balance(data.len());

    // 获取 pda 账户地址以及 bump 值
    let (pda, bump_seed) =
        Pubkey::find_program_address(&[&account_user.key.to_bytes()], program_id);

    if pda != *account_data.key {
        return Err(ProgramError::InvalidSeeds);
    }

    // 判断账户是否存在，不存在则进行创建
    if account_data.lamports() == 0 {
        invoke_signed(
            &system_instruction::create_account(
                account_user.key,
                account_data.key,
                rent_exemption,
                data.len() as u64,
                program_id,
            ),
            accounts,
            &[&[&account_user.key.to_bytes(), &[bump_seed]]],
        )?;
        account_data.data.borrow_mut().copy_from_slice(data);
        return Ok(());
    }

    // 租金补足
    if rent_exemption > account_data.lamports() {
        // 防止下溢
        let additional_lamports = rent_exemption
            .checked_sub(account_data.lamports())
            .ok_or(ProgramError::ArithmeticOverflow)?;

        solana_program::program::invoke(
            &system_instruction::transfer(account_user.key, account_data.key, additional_lamports),
            accounts,
        )?;
    }

    // 租金退款
    if rent_exemption < account_data.lamports() {
        // 防止下溢
        let excess = account_data
            .lamports()
            .checked_sub(rent_exemption)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        **account_user.lamports.borrow_mut() += excess;
        **account_data.lamports.borrow_mut() = rent_exemption;
    }

    // 重新分配空间
    account_data.realloc(data.len(), false)?;

    account_data.data.borrow_mut().copy_from_slice(data);

    Ok(())
}
