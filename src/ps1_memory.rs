//! Platform shared-memory access to DuckStation PS1 RAM.
//!
//! On Linux, finds DuckStation by scanning `/dev/shm/duckstation_<pid>`
//! and maps 8 MiB via `shm_open` + `mmap`.
//! On Windows, enumerates running processes to find DuckStation and
//! opens its named file mapping via `OpenFileMappingW` + `MapViewOfFile`.

use std::fmt;

use anyhow::{Context, Result};

use crate::protocol::OnlineCTR;

pub const LOBBY_LEVEL_ID: i8 = 38;

pub const ONLINE_CTR_OFFSET: usize = 0x8000C000 & 0xFFFFFF;
pub const SHARED_MEMORY_SIZE: usize = 0x800000;

pub const CHARACTER_ID: u32 = 0x80086e84;
pub const CHEATS: u32 = 0x80096b28;
pub const GAMEPAD_BASE: u32 = 0x80096804;
pub const GAMEMODE: u32 = 0x80096b20;
pub const LOADING_STAGE: u32 = 0x8008d0f8;
pub const PSX_POINTER: u32 = 0x8009900c;

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
        let Ok(entry) = entry else { continue };
        let fname = entry.file_name();
        let name_str = fname.to_string_lossy();
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
    use std::ffi::CString;
    use std::ptr;

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

#[cfg(windows)]
fn find_duckstation_pid() -> Option<i32> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE, TRUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::*;

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return None;
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry) == TRUE {
            loop {
                let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(260);
                let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
                if name.to_lowercase().starts_with("duckstation") {
                    let pid = entry.th32ProcessID;
                    CloseHandle(snapshot);
                    return Some(pid as i32);
                }
                if Process32NextW(snapshot, &mut entry) != TRUE {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
        None
    }
}

#[cfg(windows)]
fn open_shmem(pid: i32) -> Result<*mut u8> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Memory::*;

    let name: Vec<u16> = format!("duckstation_{}\0", pid).encode_utf16().collect();

    unsafe {
        let handle = OpenFileMappingW(FILE_MAP_READ | FILE_MAP_WRITE, 0, name.as_ptr());
        if handle.is_null() {
            return Err(Ps1MemoryError::OpenFailed(std::io::Error::last_os_error()).into());
        }

        let view = MapViewOfFile(
            handle,
            FILE_MAP_READ | FILE_MAP_WRITE,
            0,
            0,
            SHARED_MEMORY_SIZE,
        );
        CloseHandle(handle);

        if view.Value.is_null() {
            return Err(Ps1MemoryError::MapFailed(std::io::Error::last_os_error()).into());
        }

        Ok(view.Value as *mut u8)
    }
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
            #[cfg(unix)]
            unsafe {
                libc::munmap(self.pointer as *mut libc::c_void, self.size);
            }
            #[cfg(windows)]
            unsafe {
                use windows_sys::Win32::System::Memory::{
                    MEMORY_MAPPED_VIEW_ADDRESS, UnmapViewOfFile,
                };
                UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                    Value: self.pointer as *mut core::ffi::c_void,
                });
            }
        }
    }
}
