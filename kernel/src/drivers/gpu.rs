use virtio_drivers::{
    transport::{mmio::{MmioTransport, VirtIOHeader}, Transport, DeviceType},
    device::gpu::VirtIOGpu,
};
use crate::drivers::virtio::HalImpl;
use core::ptr::NonNull;
use spin::Mutex;

pub static GPU: Mutex<Option<VirtIOGpu<HalImpl, MmioTransport>>> = Mutex::new(None);
pub static FB_CONFIG: Mutex<Option<(usize, u32, u32)>> = Mutex::new(None);
static CURRENT_PROGRESS: Mutex<u32> = Mutex::new(0);

fn spin_wait(cycles: u64) {
    for _ in 0..cycles {
        unsafe { core::arch::asm!("nop"); }
    }
}

pub fn init() {
    for i in 0..32 {
        let base = 0x0a000000 + (i * 0x200);
        let header = unsafe { NonNull::new_unchecked(base as *mut VirtIOHeader) };
        if let Ok(transport) = unsafe { MmioTransport::new(header) } {
            if transport.device_type() == DeviceType::GPU {
                crate::println!("[gpu] Found VirtIO GPU at {:#x}", base);
                match VirtIOGpu::<HalImpl, _>::new(transport) {
                    Ok(mut gpu) => {
                        let (width, height) = gpu.resolution().unwrap();
                        crate::println!("[gpu] Initialized: {}x{}", width, height);
                        
                        // Set up framebuffer ONCE
                        let fb = gpu.setup_framebuffer().unwrap();
                        let fb_ptr = fb.as_mut_ptr() as usize;
                        
                        *FB_CONFIG.lock() = Some((fb_ptr, width, height));
                        *GPU.lock() = Some(gpu);
                        
                        draw_boot_screen();
                        return;
                    }
                    Err(e) => crate::println!("[gpu] Failed to initialize: {:?}", e),
                }
            }
        }
    }
}

pub fn fill_rect(fb_ptr: usize, width: u32, height: u32, x: u32, y: u32, w: u32, h: u32, color: (u8, u8, u8)) {
     let fb = unsafe { core::slice::from_raw_parts_mut(fb_ptr as *mut u8, (width * height * 4) as usize) };
     for dy in 0..h {
         for dx in 0..w {
             let px = x + dx;
             let py = y + dy;
             if px < width && py < height {
                 let idx = ((py * width + px) * 4) as usize;
                 fb[idx] = color.2; 
                 fb[idx + 1] = color.1; 
                 fb[idx + 2] = color.0; 
                 fb[idx + 3] = 255;
             }
         }
     }
}

pub fn draw_gradient(fb_ptr: usize, width: u32, height: u32) {
    let fb = unsafe { core::slice::from_raw_parts_mut(fb_ptr as *mut u8, (width * height * 4) as usize) };
    for y in 0..height {
        // Dark gray to black vertical gradient
        let ratio = y as f32 / height as f32;
        let color = (
            (20.0 * (1.0 - ratio)) as u8,
            (20.0 * (1.0 - ratio)) as u8,
            (25.0 * (1.0 - ratio)) as u8,
        );
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            fb[idx] = color.2;
            fb[idx + 1] = color.1;
            fb[idx + 2] = color.0;
            fb[idx + 3] = 255;
        }
    }
}

pub fn draw_pixel_alpha(fb_ptr: usize, width: u32, height: u32, x: u32, y: u32, color: (u8, u8, u8, u8)) {
    if x >= width || y >= height { return; }
    let fb = unsafe { core::slice::from_raw_parts_mut(fb_ptr as *mut u8, (width * height * 4) as usize) };
    let idx = ((y * width + x) * 4) as usize;
    
    let alpha = color.3 as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;
    
    fb[idx] = (color.2 as f32 * alpha + fb[idx] as f32 * inv_alpha) as u8;
    fb[idx + 1] = (color.1 as f32 * alpha + fb[idx + 1] as f32 * inv_alpha) as u8;
    fb[idx + 2] = (color.0 as f32 * alpha + fb[idx + 2] as f32 * inv_alpha) as u8;
    fb[idx + 3] = 255;
}

pub fn draw_boot_screen() {
    let mut gpu_lock = GPU.lock();
    let fb_config = FB_CONFIG.lock();
    
    if let (Some(ref mut gpu), Some((fb_ptr, width, height))) = (&mut *gpu_lock, *fb_config) {
        let logo_data = include_bytes!("../../../assets/logo.bmp");
        
        // Draw background gradient
        draw_gradient(fb_ptr, width, height);

        if logo_data.len() > 54 && &logo_data[0..2] == b"BM" {
            let offset = u32::from_le_bytes([logo_data[10], logo_data[11], logo_data[12], logo_data[13]]) as usize;
            let logo_width = i32::from_le_bytes([logo_data[18], logo_data[19], logo_data[20], logo_data[21]]) as i32;
            let logo_height = i32::from_le_bytes([logo_data[22], logo_data[23], logo_data[24], logo_data[25]]) as i32;
            
            let x_off = (width as i32 - logo_width) / 2;
            let abs_height = logo_height.abs();
            let y_off = (height as i32 - abs_height) / 2 - 50;
            let row_size = ((24 * logo_width + 31) / 32) * 4;
            
            for dy in 0..abs_height {
                for dx in 0..logo_width {
                    let y_in_bmp = if logo_height > 0 { abs_height - 1 - dy } else { dy };
                    let pixel_idx = offset + (y_in_bmp as usize * row_size as usize) + (dx as usize * 3);
                    
                    if pixel_idx + 2 < logo_data.len() {
                        let b = logo_data[pixel_idx];
                        let g = logo_data[pixel_idx + 1];
                        let r = logo_data[pixel_idx + 2];
                        
                        // Simple alpha: if it's very dark, assume it's background
                        let luma = (r as u32 + g as u32 + b as u32) / 3;
                        if luma >= 10 {
                            draw_pixel_alpha(fb_ptr, width, height, (x_off + dx) as u32, (y_off + dy) as u32, (r, g, b, 255));
                        }
                    }
                }
            }
            
            // Draw progress bar track
            let bar_width = 300;
            let bar_height = 6;
            let bar_x = (width - bar_width) / 2;
            let bar_y = (y_off + abs_height + 60) as u32;
            
            // Track (Semi-transparent dark gray)
            fill_rect(fb_ptr, width, height, bar_x, bar_y, bar_width, bar_height, (40, 40, 45));
        }
        gpu.flush().unwrap();
    }
}

pub fn update_progress(percent: u32) {
    let mut current = CURRENT_PROGRESS.lock();
    let start = *current;
    let end = if percent > 100 { 100 } else { percent };
    
    if end <= start { return; }

    let mut gpu_lock = GPU.lock();
    let fb_config = FB_CONFIG.lock();
    
    if let (Some(ref mut gpu), Some((fb_ptr, width, height))) = (&mut *gpu_lock, *fb_config) {
        let logo_h = 558; 
        let bar_width = 300;
        let bar_height = 6;
        let bar_x = (width - bar_width) / 2;
        let bar_y = (height as i32 - logo_h) / 2 - 50 + logo_h + 60;

        for p in start..=end {
            let progress_width = (bar_width * p) / 100;
            
            // Draw progress bar for current percentage
            for dx in 0..progress_width {
                 for dy in 0..bar_height {
                     draw_pixel_alpha(fb_ptr, width, height, bar_x + dx, bar_y as u32 + dy, (255, 255, 255, 255));
                 }
            }
            
            // Add a subtle glow at the tip
            if progress_width > 0 && progress_width < bar_width {
                let tip_x = bar_x + progress_width;
                let tip_y = bar_y as u32 + (bar_height / 2);
                for i in 1..8 {
                    let alpha = (128 / (i * 2)) as u8;
                    draw_pixel_alpha(fb_ptr, width, height, tip_x, tip_y, (255, 255, 255, alpha));
                }
            }
            
            gpu.flush().unwrap();
            
            // Subtle delay for animation effect
            spin_wait(1_000_000); 
        }
    }
    
    *current = end;
}
