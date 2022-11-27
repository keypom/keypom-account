use crate::*;

/// debugging only
use near_sys::log_utf8;
pub fn log(message: &str) {
    unsafe {
        log_utf8(message.len() as _, message.as_ptr() as _);
    }
}

pub fn read_register_fixed<const N: usize>(register_id: u64) -> [u8; N] {
    let mut buffer = [0; N];
    //* SAFETY: Assumes register length is not greater than the buffer. Less is fine
    //* 		since the buffer is zeroed, but more will cause UB.
    unsafe { near_sys::read_register(register_id, buffer.as_mut_ptr() as _) };
    buffer
}

pub(crate) unsafe fn return_bytes(bytes: &[u8], json: bool) {
    let mut ret_data = vec![DOUBLE_QUOTE_BYTE];
    if json == true {
        let bytes_str = alloc::str::from_utf8(&bytes).ok().unwrap_or_else(|| sys::panic());
        ret_data.extend_from_slice(bytes_str
            .replace("\"", "\\\"")
            .replace("|kP|", "")
            .replace("|kS|", "")
            .as_bytes()
        );
    } else {
        ret_data.extend_from_slice(bytes);
    }
    ret_data.push(DOUBLE_QUOTE_BYTE);
    near_sys::value_return(ret_data.len() as u64, ret_data.as_ptr() as u64);
}

pub(crate) fn swrite(key: &[u8], val: &[u8]) {
    //* SAFETY: Assumes valid storage_write implementation.
    unsafe {
        near_sys::storage_write(
            key.len() as u64,
            key.as_ptr() as u64,
            val.len() as u64,
            val.as_ptr() as u64,
            REGISTER_0,
        );
    }
}

pub(crate) fn storage_read(key: &[u8]) -> Vec<u8> {
    let key_exists =
        unsafe { near_sys::storage_read(key.len() as u64, key.as_ptr() as u64, REGISTER_0) };
    if key_exists == 0 {
        // Return code of 0 means storage key had no entry.
        sys::panic()
    }
    register_read(REGISTER_0)
}

//* SAFETY: Assumes that length of storage value at this key is less than u64 buffer len (8).
pub(crate) unsafe fn sread_u64(key: &[u8]) -> u64 {
    near_sys::storage_read(key.len() as u64, key.as_ptr() as u64, REGISTER_0);
    u64::from_le_bytes(read_register_fixed(REGISTER_0))
}

pub(crate) fn register_read(id: u64) -> Vec<u8> {
    let len = unsafe { near_sys::register_len(id) };
    if len == u64::MAX {
        // Register was not found
        sys::panic()
    }
    let data = vec![0u8; len as usize];

    //* SAFETY: Length of buffer is set dynamically based on `register_len` so it will always
    //* 		be sufficient length.
    unsafe { near_sys::read_register(id, data.as_ptr() as u64) };
    data
}

pub(crate) fn panic() -> ! {
    //* SAFETY: Assumed valid panic host function implementation
    unsafe { near_sys::panic() }
}
