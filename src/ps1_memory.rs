use std::ffi::CString;
use std::fmt;
use std::ptr;

use anyhow::{Context, Result};

use crate::protocol::OnlineCTR;

pub const LOBBY_LEVEL_ID: i8 = 38;

pub const SHARED_MEMORY_SIZE: usize = 0x800000;
pub const ONLINE_CTR_OFFSET: usize = 0x8000C000 & 0xFFFFFF;

pub const GAMEPAD_BASE: u32 = 0x80096804;
pub const GAMEMODE: u32 = 0x80096b20;
pub const LOADING_STAGE: u32 = 0x8008d0f8;

#[derive(Debug)]
pub enum Ps1MemoryError {
    NotFound,
    OpenFailed(std::io::Error),
    MapFailed(std::io::Error),
    OutOfBounds { address: u32, requested: usize },
}

impl fmt::Display for Ps1MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ps1MemoryError::NotFound => write!(f, "DuckStation is not running"),
            Ps1MemoryError::OpenFailed(e) => write!(f, "failed to open shared memory: {e}"),
            Ps1MemoryError::MapFailed(e) => write!(f, "failed to map shared memory: {e}"),
            Ps1MemoryError::OutOfBounds { address, requested } => {
                write!(
                    f,
                    "address 0x{address:08X} (size {requested}B) is out of bounds"
                )
            }
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
        if let Some(pid_str) = name_str.strip_prefix("duckstation_")
            && let Ok(pid) = pid_str.parse::<i32>()
        {
            return Some(pid);
        }
    }
    None
}

#[cfg(unix)]
fn open_shmem(pid: i32) -> Result<*mut u8> {
    let name = CString::new(format!("duckstation_{}", pid))?;

    let file_descriptor = unsafe { libc::shm_open(name.as_ptr(), libc::O_RDWR, 0o600) };
    if file_descriptor < 0 {
        return Err(Ps1MemoryError::OpenFailed(std::io::Error::last_os_error()).into());
    }

    let pointer = unsafe {
        let p = libc::mmap(
            ptr::null_mut(),
            SHARED_MEMORY_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            file_descriptor,
            0,
        );
        libc::close(file_descriptor);

        if p == libc::MAP_FAILED {
            return Err(Ps1MemoryError::MapFailed(std::io::Error::last_os_error()).into());
        }
        p as *mut u8
    };

    Ok(pointer)
}

pub struct Ps1Memory {
    pointer: *mut u8,
    size: usize,
}

impl Ps1Memory {
    pub fn connect() -> Result<Self> {
        let pid = find_duckstation_pid()
            .ok_or(Ps1MemoryError::NotFound)
            .context("waiting for DuckStation, make sure shared memory is enabled")?;
        let pointer =
            open_shmem(pid).context("failed to access DuckStation shared memory segment")?;

        Ok(Self {
            pointer,
            size: SHARED_MEMORY_SIZE,
        })
    }

    fn get_offset(&self, address: u32, size_of_type: usize) -> Result<usize> {
        let offset = (address & 0xFFFFFF) as usize;
        if offset + size_of_type > self.size {
            return Err(Ps1MemoryError::OutOfBounds {
                address,
                requested: size_of_type,
            }
            .into());
        }
        Ok(offset)
    }

    pub fn online_ctr(&self) -> &OnlineCTR {
        unsafe { &*(self.pointer.add(ONLINE_CTR_OFFSET) as *const OnlineCTR) }
    }

    pub fn online_ctr_mut(&mut self) -> &mut OnlineCTR {
        unsafe { &mut *(self.pointer.add(ONLINE_CTR_OFFSET) as *mut OnlineCTR) }
    }

    pub fn read_u8(&self, address: u32) -> Result<u8> {
        let offset = self.get_offset(address, 1)?;
        unsafe { Ok(*self.pointer.add(offset)) }
    }

    pub fn read_u16(&self, address: u32) -> Result<u16> {
        let offset = self.get_offset(address, 2)?;
        unsafe { Ok((self.pointer.add(offset) as *const u16).read_unaligned()) }
    }

    pub fn read_u32(&self, address: u32) -> Result<u32> {
        let offset = self.get_offset(address, 4)?;
        unsafe { Ok((self.pointer.add(offset) as *const u32).read_unaligned()) }
    }

    pub fn write_u8(&mut self, address: u32, val: u8) -> Result<()> {
        let offset = self.get_offset(address, 1)?;
        unsafe { *self.pointer.add(offset) = val };
        Ok(())
    }

    pub fn write_u16(&mut self, address: u32, val: u16) -> Result<()> {
        let offset = self.get_offset(address, 2)?;
        unsafe { (self.pointer.add(offset) as *mut u16).write_unaligned(val) };
        Ok(())
    }

    pub fn write_u32(&mut self, address: u32, val: u32) -> Result<()> {
        let offset = self.get_offset(address, 4)?;
        unsafe { (self.pointer.add(offset) as *mut u32).write_unaligned(val) };
        Ok(())
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
