// touhou-rpc-for-linux — Native Linux Discord Rich Presence for the Touhou series.
// Copyright (C) 2026 SandwichVertigo
//
// This program is a derivative work of TouhouRPC by TheBakaRem (GPL-3.0-or-later).
// See NOTICE for details on what was derived and what is original.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Memory reader over `process_vm_readv(2)` with `/proc/<pid>/mem` fallback.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use anyhow::{bail, Context, Result};

pub struct MemoryReader {
    pid: i32,
    mem_file: Option<File>,
    use_fallback: bool,
}

impl MemoryReader {
    pub fn new(pid: i32) -> Self {
        Self { pid, mem_file: None, use_fallback: false }
    }

    pub fn pid(&self) -> i32 { self.pid }

    pub fn read(&mut self, addr: u64, buf: &mut [u8]) -> Result<()> {
        if !self.use_fallback {
            match self.read_vm(addr, buf) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    let errno = e.downcast_ref::<std::io::Error>().and_then(|io| io.raw_os_error());
                    if matches!(errno, Some(libc::EPERM) | Some(libc::EACCES)) {
                        self.use_fallback = true;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        self.read_proc_mem(addr, buf)
    }

    pub fn u8(&mut self, addr: u64) -> Result<u8> {
        let mut b = [0u8; 1]; self.read(addr, &mut b)?; Ok(b[0])
    }
    pub fn u16(&mut self, addr: u64) -> Result<u16> {
        let mut b = [0u8; 2]; self.read(addr, &mut b)?; Ok(u16::from_le_bytes(b))
    }
    pub fn u32(&mut self, addr: u64) -> Result<u32> {
        let mut b = [0u8; 4]; self.read(addr, &mut b)?; Ok(u32::from_le_bytes(b))
    }
    pub fn i32(&mut self, addr: u64) -> Result<i32> {
        let mut b = [0u8; 4]; self.read(addr, &mut b)?; Ok(i32::from_le_bytes(b))
    }
    pub fn f32(&mut self, addr: u64) -> Result<f32> {
        let mut b = [0u8; 4]; self.read(addr, &mut b)?; Ok(f32::from_le_bytes(b))
    }
    /// Read a NUL-terminated ASCII string, up to `max_len` bytes.
    pub fn cstr(&mut self, addr: u64, max_len: usize) -> Result<String> {
        let mut buf = vec![0u8; max_len];
        self.read(addr, &mut buf)?;
        let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        Ok(String::from_utf8_lossy(&buf[..end]).into_owned())
    }

    fn read_vm(&self, addr: u64, buf: &mut [u8]) -> Result<()> {
        let local = libc::iovec { iov_base: buf.as_mut_ptr() as *mut _, iov_len: buf.len() };
        let remote = libc::iovec { iov_base: addr as *mut _, iov_len: buf.len() };
        let n = unsafe {
            libc::process_vm_readv(self.pid as libc::pid_t, &local, 1, &remote, 1, 0)
        };
        if n < 0 { return Err(std::io::Error::last_os_error()).context("process_vm_readv"); }
        if (n as usize) != buf.len() { bail!("short read: {}/{}", n, buf.len()); }
        Ok(())
    }

    fn read_proc_mem(&mut self, addr: u64, buf: &mut [u8]) -> Result<()> {
        if self.mem_file.is_none() {
            let path = format!("/proc/{}/mem", self.pid);
            self.mem_file = Some(File::open(&path).with_context(|| format!("open {}", path))?);
        }
        let f = self.mem_file.as_mut().unwrap();
        f.seek(SeekFrom::Start(addr))?;
        f.read_exact(buf)?;
        Ok(())
    }
}
