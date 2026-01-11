use alloc::string::String;
use alloc::string::ToString;

// Embed the RAM Disk
// Use include_bytes! to load disk.tar from project root.
pub static RAMDISK: &[u8] = include_bytes!("../../disk.tar");

/// A file entry in the Tar filesystem
#[derive(Debug, Clone)]
pub struct FileEntry<'a> {
    pub name: String,
    pub size: usize,
    pub data: &'a [u8],
    pub is_dir: bool,
}

/// Iterate over files in a TAR archive
pub struct TarIterator<'a> {
    archive: &'a [u8],
    offset: usize,
}

impl<'a> TarIterator<'a> {
    pub fn new(archive: &'a [u8]) -> Self {
        Self { archive, offset: 0 }
    }
}

impl<'a> Iterator for TarIterator<'a> {
    type Item = FileEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Check for End of Archive (two empty 512 blocks)
        if self.offset + 512 > self.archive.len() {
            return None;
        }

        let header = &self.archive[self.offset..self.offset + 512];
        
        // Check if header is zero (end marker)
        if header.iter().all(|&b| b == 0) {
            return None;
        }

        // Parse Name (0..100)
        let name_bytes = &header[0..100];
        let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(100);
        let name = core::str::from_utf8(&name_bytes[0..name_len]).unwrap_or("unknown").to_string(); // Need alloc::string::ToString

        // Parse Size (124..136) - Octal ASCII
        let size_bytes = &header[124..136];
        let size_len = size_bytes.iter().position(|&b| b == 0).unwrap_or(12);
        let size_str = core::str::from_utf8(&size_bytes[0..size_len]).unwrap_or("0");
        let size = usize::from_str_radix(size_str.trim(), 8).unwrap_or(0);

        // Type Flag (156)
        let type_flag = header[156];
        let is_dir = type_flag == b'5';

        // Data starts after header (512 bytes)
        let data_start = self.offset + 512;
        let data_end = data_start + size;
        
        if data_end > self.archive.len() {
            return None; // Invalid or truncated
        }

        let data = &self.archive[data_start..data_end];

        // Advance offset. Data is rounded up to 512 bytes.
        let block_size = (size + 511) / 512 * 512;
        self.offset += 512 + block_size;

        Some(FileEntry { name, size, data, is_dir })
    }
}

pub fn ls(archive: &[u8]) {
    use aprk_arch_arm64::println;
    
    println!("Type  Size    Name");
    println!("----  ------  ----");
    
    for file in TarIterator::new(archive) {
        let type_char = if file.is_dir { 'd' } else { 'f' };
        println!("{}     {: <6}  {}", type_char, file.size, file.name);
    }
}

pub fn get_file<'a>(archive: &'a [u8], name: &str) -> Option<FileEntry<'a>> {
    for file in TarIterator::new(archive) {
        if file.name == name {
            return Some(file);
        }
    }
    None
}
