#![cfg_attr(target_os = "solana", no_std)]

pub const STORAGE_SIZE: usize = 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageError {
    TooLarge,
}

#[inline]
pub fn set_return_data_u64(value: u64) {
    pinocchio::program::set_return_data(&value.to_le_bytes());
}

#[inline]
pub fn set_storage(storage: &[u8]) -> Result<(), StorageError> {
    if storage.len() > STORAGE_SIZE {
        return Err(StorageError::TooLarge);
    }
    #[cfg(target_os = "solana")]
    unsafe {
        sol_set_storage(storage.as_ptr(), storage.len() as u64);
    }
    Ok(())
}

#[cfg(target_os = "solana")]
extern "C" {
    fn sol_set_storage(data: *const u8, length: u64);
}
