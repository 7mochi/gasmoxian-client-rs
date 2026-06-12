use std::error::Error;

use crate::protocol::OnlineCTR;

pub const SHMEM_SIZE: usize = 0x800000; // RAM size of the PS1
pub const OCTR_OFFSET: usize = 0x8000C000 & 0xFFFFFF;

pub const ADDR_CHARACTER_ID: u32 = 0x80086e84;
pub const ADDR_GAMEPAD_BASE: u32 = 0x80096804;
pub const ADDR_PSX_PTR: u32 = 0x8009900c;
pub const ADDR_CHEATS: u32 = 0x80096b28;
pub const ADDR_LOADING_STAGE: u32 = 0x8008d0f8;
pub const ADDR_GAME_MODE: u32 = 0x80096b20;

#[cfg(unix)]
fn find_duckstation_pid() -> Option<i32> {
    let dir = std::fs::read_dir("/dev/shm").ok()?;
    for entry in dir {
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
fn open_shmem(pid: i32) -> Result<*mut u8, Box<dyn Error>> {
    let name = format!("duckstation_{}\0", pid);
    let fd = unsafe { libc::shm_open(name.as_ptr() as *const i8, libc::O_RDWR, 0o600) };
    if fd < 0 {
        return Err(Box::new(std::io::Error::last_os_error()));
    }
    let ptr = unsafe {
        let p = libc::mmap(
            std::ptr::null_mut(),
            SHMEM_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        );
        if p == libc::MAP_FAILED {
            libc::close(fd);
            return Err(Box::new(std::io::Error::last_os_error()));
        }
        libc::close(fd);
        p as *mut u8
    };
    Ok(ptr)
}

pub struct Ps1Mem {
    pointer: *mut u8,
    size: usize,
}

impl Ps1Mem {
    pub fn connect() -> Result<Self, Box<dyn Error>> {
        let pid = find_duckstation_pid()
            .ok_or_else(|| Box::<dyn Error>::from("DuckStation is not running"))?;
        let ptr = open_shmem(pid)?;
        Ok(Ps1Mem {
            pointer: ptr,
            size: SHMEM_SIZE,
        })
    }
    pub fn octr(&self) -> &OnlineCTR {
        unsafe { &*(self.pointer.add(OCTR_OFFSET) as *const OnlineCTR) }
    }

    pub fn octr_mut(&self) -> &mut OnlineCTR {
        unsafe { &mut *(self.pointer.add(OCTR_OFFSET) as *mut OnlineCTR) }
    }

    pub fn write_u16(&self, addr: u32, val: u16) {
        unsafe { *(self.pointer.add((addr & 0xFFFFFF) as usize) as *mut u16) = val }
    }

    pub fn read_u8(&self, addr: u32) -> u8 {
        unsafe { *self.pointer.add((addr & 0xFFFFFF) as usize) }
    }

    pub fn write_u8(&self, addr: u32, val: u8) {
        unsafe { *self.pointer.add((addr & 0xFFFFFF) as usize) = val }
    }

    pub fn read_u16(&self, addr: u32) -> u16 {
        unsafe { *(self.pointer.add((addr & 0xFFFFFF) as usize) as *const u16) }
    }

    pub fn read_u32(&self, addr: u32) -> u32 {
        unsafe { *(self.pointer.add((addr & 0xFFFFFF) as usize) as *const u32) }
    }

    pub fn write_u32(&self, addr: u32, val: u32) {
        unsafe { *(self.pointer.add((addr & 0xFFFFFF) as usize) as *mut u32) = val }
    }
}

impl Drop for Ps1Mem {
    fn drop(&mut self) {
        if !self.pointer.is_null() {
            unsafe {
                libc::munmap(self.pointer as *mut libc::c_void, self.size);
            }
        }
    }
}
