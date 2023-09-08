use thiserror::Error;

#[derive(Debug, Error)]
pub enum RashinErr {
    #[error("Syscall returns some error. errno = {0}")]
    SyscallError(i32),
}
