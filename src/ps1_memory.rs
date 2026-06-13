use std::fmt;

use anyhow::{Context, Result};

use crate::protocol::OnlineCTR;

pub const SHARED_MEMORY_SIZE: usize = 0x800000;
pub const ONLINE_CTR_OFFSET: usize = 0x8000C000 & 0xFFFFFF;

pub const PSX_POINTER: u32 = 0x8009900c;
pub const CHARACTER_ID: u32 = 0x80086e84;
pub const GAMEPAD_BASE: u32 = 0x80096804;
pub const CHEATS: u32 = 0x80096b28;
pub const LOADING_STAGE: u32 = 0x8008d0f8;
pub const GAME_MODE: u32 = 0x80096b20;

#[derive(Debug)]
pub enum Ps1MemoryError {
    NotFound,
    OpenFailed(String),
    MapFailed(String),
}

impl fmt::Display for Ps1MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ps1MemoryError::NotFound => write!(f, "DuckStation is not running"),
            Ps1MemoryError::OpenFailed(e) => write!(f, "failed to open shared memory: {e}"),
            Ps1MemoryError::MapFailed(e) => write!(f, "failed to map shared memory: {e}"),
        }
    }
}

impl std::error::Error for Ps1MemoryError {}

#[cfg(unix)]
fn find_duckstation_pid() -> Option<i32> {
    let directory = std::fs::read_dir("/dev/shm").ok()?;
    for entry in directory {
        let name = entry.ok()?.file_name();
        let name_str = name.to_string_lossy();
        if let Some(pid_str) = name_str.strip_prefix("duckstation_") {
            if let Ok(pid) = pid_str.parse::<i32>() {
                return Some(pid);
            }
        }
    }
    None
}

#[cfg(unix)]
fn open_shmem(pid: i32) -> Result<*mut u8> {
    let name = format!("duckstation_{}\0", pid);
    let fd = unsafe { libc::shm_open(name.as_ptr() as *const i8, libc::O_RDWR, 0o600) };
    if fd < 0 {
        return Err(Ps1MemoryError::OpenFailed(std::io::Error::last_os_error().to_string()).into());
    }
    let ptr = unsafe {
        let p = libc::mmap(
            std::ptr::null_mut(),
            SHARED_MEMORY_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        );
        if p == libc::MAP_FAILED {
            libc::close(fd);
            return Err(
                Ps1MemoryError::MapFailed(std::io::Error::last_os_error().to_string()).into(),
            );
        }
        libc::close(fd);
        p as *mut u8
    };
    Ok(ptr)
}

pub struct Ps1Memory {
    pointer: *mut u8,
    size: usize,
}

impl Ps1Memory {
    /// Connects to DuckStation's shared memory.
    ///
    /// Scans `/dev/shm/duckstation_*` for a running DuckStation process,
    /// then opens and maps the 8 MB shared memory segment.
    ///
    /// # Errors
    /// Returns [`Ps1MemoryError::NotFound`] if no DuckStation is detected,
    /// or [`Ps1MemoryError::OpenFailed`]/[`Ps1MemoryError::MapFailed`]
    /// if shared memory access fails.
    pub fn connect() -> Result<Self> {
        let pid = find_duckstation_pid()
            .ok_or(Ps1MemoryError::NotFound)
            .context("waiting for DuckStation, make sure shared memory is enabled")?;
        let pointer =
            open_shmem(pid).context("failed to access DuckStation shared memory segment")?;
        Ok(Ps1Memory {
            pointer,
            size: SHARED_MEMORY_SIZE,
        })
    }

    pub fn online_ctr(&self) -> &OnlineCTR {
        unsafe { &*(self.pointer.add(ONLINE_CTR_OFFSET) as *const OnlineCTR) }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn online_ctr_mut(&self) -> &mut OnlineCTR {
        unsafe { &mut *(self.pointer.add(ONLINE_CTR_OFFSET) as *mut OnlineCTR) }
    }

    pub fn write_u8(&self, address: u32, val: u8) {
        unsafe { *self.pointer.add((address & 0xFFFFFF) as usize) = val }
    }

    pub fn write_u16(&self, address: u32, val: u16) {
        unsafe { *(self.pointer.add((address & 0xFFFFFF) as usize) as *mut u16) = val }
    }

    pub fn write_u32(&self, address: u32, val: u32) {
        unsafe { *(self.pointer.add((address & 0xFFFFFF) as usize) as *mut u32) = val }
    }

    pub fn read_u8(&self, address: u32) -> u8 {
        unsafe { *self.pointer.add((address & 0xFFFFFF) as usize) }
    }

    pub fn read_u16(&self, address: u32) -> u16 {
        unsafe { *(self.pointer.add((address & 0xFFFFFF) as usize) as *const u16) }
    }

    pub fn read_u32(&self, address: u32) -> u32 {
        unsafe { *(self.pointer.add((address & 0xFFFFFF) as usize) as *const u32) }
    }
}

impl Drop for Ps1Memory {
    fn drop(&mut self) {
        if !self.pointer.is_null() {
            unsafe {
                libc::munmap(self.pointer as *mut libc::c_void, self.size);
            }
        }
    }
}
