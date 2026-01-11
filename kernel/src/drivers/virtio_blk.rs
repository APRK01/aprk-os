use virtio_drivers::{
    transport::{mmio::{MmioTransport, VirtIOHeader}, Transport, DeviceType},
    device::blk::VirtIOBlk,
};
use crate::drivers::virtio::HalImpl;
use core::ptr::NonNull;
use spin::Mutex;
use alloc::vec::Vec;
use alloc::vec;

pub static BLK: Mutex<Option<VirtIOBlk<HalImpl, MmioTransport>>> = Mutex::new(None);

pub fn init() {
    for i in 0..32 {
        let base = 0x0a000000 + (i * 0x200);
        let header = unsafe { NonNull::new_unchecked(base as *mut VirtIOHeader) };
        if let Ok(transport) = unsafe { MmioTransport::new(header) } {
            let dev_type = transport.device_type();
            if dev_type != DeviceType::Invalid {
                crate::println!("[blk] Found VirtIO device type {:?} at {:#x}", dev_type, base);
            }
            if dev_type == DeviceType::Block {
                crate::println!("[blk] Initializing VirtIO Block...");
                match VirtIOBlk::<HalImpl, _>::new(transport) {
                    Ok(blk) => {
                        crate::println!("[blk] Initialized. Capacity: {} sectors", blk.capacity());
                        *BLK.lock() = Some(blk);
                        return;
                    }
                    Err(e) => crate::println!("[blk] Failed to initialize: {:?}", e),
                }
            }
        }
    }
}

pub fn read_block(block_id: usize, buf: &mut [u8]) -> Result<(), ()> {
    let mut blk_lock = BLK.lock();
    if let Some(ref mut blk) = *blk_lock {
        match blk.read_blocks(block_id, buf) {
            Ok(_) => Ok(()),
            Err(e) => {
                crate::println!("[blk] Read error at {}: {:?}", block_id, e);
                Err(())
            }
        }
    } else {
        Err(())
    }
}

pub fn write_block(block_id: usize, buf: &[u8]) -> Result<(), ()> {
    let mut blk_lock = BLK.lock();
    if let Some(ref mut blk) = *blk_lock {
        match blk.write_blocks(block_id, buf) {
            Ok(_) => Ok(()),
            Err(e) => {
                crate::println!("[blk] Write error at {}: {:?}", block_id, e);
                Err(())
            }
        }
    } else {
        Err(())
    }
}
