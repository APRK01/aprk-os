pub mod gpu;
pub mod virtio;
pub mod virtio_blk;

pub fn init() {
    virtio::init();
    gpu::init();
    virtio_blk::init();
}
