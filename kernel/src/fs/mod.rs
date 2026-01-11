use alloc::sync::Arc;
use spin::Mutex;
use fatfs::{FileSystem, FsOptions, SeekFrom, Read};
use crate::drivers::virtio_blk;

pub struct BlockDeviceWrapper;

impl fatfs::IoBase for BlockDeviceWrapper {
    type Error = ();
}

impl fatfs::Read for BlockDeviceWrapper {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
        Err(())
    }
}

// better approach: implement a seekable wrapper that tracks offset
pub struct SeekableBlockDevice {
    offset: u64,
}

impl SeekableBlockDevice {
    pub fn new() -> Self {
        Self { offset: 0 }
    }
}

impl fatfs::IoBase for SeekableBlockDevice {
    type Error = ();
}

impl fatfs::Read for SeekableBlockDevice {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut read_bytes = 0;
        let block_size = 512u64;
        
        while read_bytes < buf.len() {
            let start_block = (self.offset / block_size) as usize;
            let offset_in_block = (self.offset % block_size) as usize;
            
            let mut temp_buf = [0u8; 512];
            virtio_blk::read_block(start_block, &mut temp_buf)?;
            
            let remaining_in_block = block_size as usize - offset_in_block;
            let remaining_in_buf = buf.len() - read_bytes;
            let to_copy = core::cmp::min(remaining_in_block, remaining_in_buf);
            
            buf[read_bytes..read_bytes + to_copy].copy_from_slice(&temp_buf[offset_in_block..offset_in_block + to_copy]);
            
            read_bytes += to_copy;
            self.offset += to_copy as u64;
            
            // If we didn't fill the block, we are done
            if to_copy < remaining_in_block { break; }
        }
        
        Ok(read_bytes)
    }
}

impl fatfs::Seek for SeekableBlockDevice {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        match pos {
            SeekFrom::Start(off) => self.offset = off,
            SeekFrom::Current(off) => self.offset = (self.offset as i64 + off) as u64,
            SeekFrom::End(_off) => {
                return Err(());
            }
        }
        Ok(self.offset)
    }
}

impl fatfs::Write for SeekableBlockDevice {
    fn write(&mut self, _buf: &[u8]) -> Result<usize, Self::Error> {
        Err(())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub static FS: Mutex<Option<FileSystem<SeekableBlockDevice, fatfs::DefaultTimeProvider, fatfs::LossyOemCpConverter>>> = Mutex::new(None);

pub fn init() {
    let dev = SeekableBlockDevice::new();
    match FileSystem::new(dev, FsOptions::new()) {
        Ok(fs) => {
            crate::println!("[fs] FAT32 FileSystem initialized.");
            *FS.lock() = Some(fs);
        }
        Err(e) => {
            crate::println!("[fs] Failed to initialize FileSystem: {:?}", e);
        }
    }
}

pub fn list_root() {
    if let Some(ref fs) = *FS.lock() {
        let root = fs.root_dir();
        crate::println!("[fs] Root directory content:");
        for entry in root.iter() {
            let entry = entry.unwrap();
            crate::println!("  {} ({})", entry.file_name(), if entry.is_dir() { "DIR" } else { "FILE" });
        }
    }
}

pub fn read_file(path: &str) -> Option<alloc::vec::Vec<u8>> {
    if let Some(ref fs) = *FS.lock() {
        let root = fs.root_dir();
        match root.open_file(path) {
            Ok(mut file) => {
                let mut buf = alloc::vec::Vec::new();
                let mut chunk = [0u8; 512];
                while let Ok(n) = file.read(&mut chunk) {
                    if n == 0 { break; }
                    buf.extend_from_slice(&chunk[..n]);
                }
                Some(buf)
            }
            Err(_) => None,
        }
    } else {
        None
    }
}
