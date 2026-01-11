use core::ptr;
use aprk_arch_arm64::{println, cpu};

#[repr(C)]
#[derive(Debug)]
struct ElfHeader {
    magic: [u8; 4],
    class: u8,
    data: u8,
    version: u8,
    osabi: u8,
    abiversion: u8,
    pad: [u8; 7],
    type_: u16,
    machine: u16,
    version2: u32,
    entry: u64,
    phoff: u64,
    shoff: u64,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

#[repr(C)]
#[derive(Debug)]
struct ProgramHeader {
    type_: u32,
    flags: u32,
    offset: u64,
    vaddr: u64,
    paddr: u64,
    filesz: u64,
    memsz: u64,
    align: u64,
}

const PT_LOAD: u32 = 1;

/// Load an ELF binary into memory.
/// Returns the Entry Point address.
pub unsafe fn load_elf(data: &[u8]) -> Option<u64> {
    if data.len() < core::mem::size_of::<ElfHeader>() {
         println!("[loader] File too small");
         return None;
    }

    // Read header manually to guarantee no alignment issues
    let mut header = core::mem::MaybeUninit::<ElfHeader>::uninit();
    ptr::copy_nonoverlapping(
        data.as_ptr(), 
        header.as_mut_ptr() as *mut u8, 
        core::mem::size_of::<ElfHeader>()
    );
    let header = header.assume_init();

    // Validate Magic (0x7F, 'E', 'L', 'F')
    if header.magic != [0x7f, 0x45, 0x4c, 0x46] {
        println!("[loader] Invalid ELF Magic");
        return None;
    }
    
    // Check Architecture (0xB7 = AArch64) -> 183 decimal
    if header.machine != 183 {
         println!("[loader] Wrong Architecture: {}", header.machine);
         return None;
    }

    println!("[loader] Loading ELF at Entry: {:#x}", header.entry);

    // Iterate Program Headers
    let ph_table = data.as_ptr().add(header.phoff as usize);
    let ent_size = header.phentsize as usize;
    
    for i in 0..header.phnum {
        let ph_ptr = ph_table.add((i as usize) * ent_size);
        
        // Manual copy for Program Header
        let mut ph = core::mem::MaybeUninit::<ProgramHeader>::uninit();
        ptr::copy_nonoverlapping(
            ph_ptr, 
            ph.as_mut_ptr() as *mut u8, 
            core::mem::size_of::<ProgramHeader>()
        );
        let ph = ph.assume_init();
        
        if ph.type_ == PT_LOAD {
            // Check if Mem Size is 0 (useless segment)
            if ph.memsz == 0 { continue; }

            // println!("[loader] Segment: VAddr {:#x}, Size {:#x}", ph.vaddr, ph.memsz);
            
            // Destination in Memory
            let dest = ph.vaddr as *mut u8;
            
            // Source in File
            let src = data.as_ptr().add(ph.offset as usize);
            
            // Size present in file
            let file_size = ph.filesz as usize;
            
            // Total size in memory
            let mem_size = ph.memsz as usize;
            
            // 1. Copy file data
            if file_size > 0 {
                ptr::copy_nonoverlapping(src, dest, file_size);
            }
            
            // 2. Zero remaining memory (BSS)
            if mem_size > file_size {
                let bss_dest = dest.add(file_size);
                let bss_size = mem_size - file_size;
                ptr::write_bytes(bss_dest, 0, bss_size);
            }
            
            // 3. Clean D-Cache for this segment to ensure visibility to I-Cache
            cpu::clean_dcache_range(dest as usize, mem_size);
        }
    }

    // Flush Cache to ensure instructions are visible
    cpu::flush_instruction_cache();

    Some(header.entry)
}
