// =============================================================================
// APRK OS - PL011 UART Driver
// =============================================================================
// Driver for the ARM PL011 UART (Universal Asynchronous Receiver/Transmitter).
// This is the primary serial console used by QEMU's virt machine.
//
// The PL011 is a fully-featured UART with FIFOs and modem control signals.
// For v0.0.1, we only implement basic transmit functionality.
//
// Reference: ARM PrimeCell UART (PL011) Technical Reference Manual
// =============================================================================

use core::fmt::{self, Write};
use spin::Mutex;

// =============================================================================
// PL011 Register Definitions
// =============================================================================

/// Base address of UART0 on QEMU virt machine
const UART0_BASE: usize = 0x0900_0000;

/// UART Register Offsets from base address
mod regs {
    /// Data Register - read/write data here
    pub const DR: usize = 0x00;
    
    /// Flag Register - contains UART status flags
    pub const FR: usize = 0x18;
    
    /// Integer Baud Rate Register
    pub const IBRD: usize = 0x24;
    
    /// Fractional Baud Rate Register
    pub const FBRD: usize = 0x28;
    
    /// Line Control Register
    pub const LCR_H: usize = 0x2C;
    
    /// Control Register
    pub const CR: usize = 0x30;
    
    /// Interrupt Mask Set/Clear Register
    pub const IMSC: usize = 0x38;

    /// Interrupt Clear Register
    pub const ICR: usize = 0x44;
}

/// Flag Register bits
mod flags {
    /// Transmit FIFO full
    pub const TXFF: u32 = 1 << 5;
    
    /// Receive FIFO empty
    pub const RXFE: u32 = 1 << 4;
}



/// Line Control Register bits
mod lcr {
    /// Enable FIFOs
    pub const FEN: u32 = 1 << 4;
    
    /// Word length: 8 bits (bits 5-6 = 0b11)
    pub const WLEN_8: u32 = 0b11 << 5;
}

/// Control Register bits
mod cr {
    /// UART enable
    pub const UARTEN: u32 = 1 << 0;
    
    /// Transmit enable
    pub const TXE: u32 = 1 << 8;
    
    /// Receive enable
    pub const RXE: u32 = 1 << 9;
}

// =============================================================================
// UART Driver Implementation
// =============================================================================

/// PL011 UART driver
pub struct Uart {
    base: usize,
}

impl Uart {
    /// Create a new UART driver instance.
    /// 
    /// # Arguments
    /// * `base` - Base address of the UART registers
    /// 
    /// # Safety
    /// The caller must ensure the base address points to valid UART hardware.
    pub const fn new(base: usize) -> Self {
        Self { base }
    }

    /// Read a register at the given offset
    fn read_reg(&self, offset: usize) -> u32 {
        let addr = (self.base + offset) as *const u32;
        // SAFETY: We trust that self.base points to valid UART registers
        unsafe { core::ptr::read_volatile(addr) }
    }

    /// Write a value to a register at the given offset
    fn write_reg(&self, offset: usize, value: u32) {
        let addr = (self.base + offset) as *mut u32;
        // SAFETY: We trust that self.base points to valid UART registers
        unsafe { core::ptr::write_volatile(addr, value) }
    }

    /// Initialize the UART.
    /// 
    /// Configures the UART for 8-N-1 operation (8 data bits, no parity, 1 stop bit).
    /// QEMU doesn't require baud rate setup, but we set it anyway for completeness.
    pub fn init(&self) {
        // Disable UART while configuring
        self.write_reg(regs::CR, 0);

        // Clear all pending interrupts
        self.write_reg(regs::IMSC, 0);
        self.write_reg(regs::ICR, 0x7FF); // Clear all interrupts

        // Enable Receive Interrupt (RXIM) and Receive Timeout (RTIM)
        // self.write_reg(regs::IMSC, imsc::RXIM | imsc::RTIM);

        // Set baud rate (115200 with 24MHz clock)
        // Divider = 24000000 / (16 * 115200) = 13.0208
        // Integer part = 13
        // Fractional part = 0.0208 * 64 = 1.33 ≈ 1
        self.write_reg(regs::IBRD, 13);
        self.write_reg(regs::FBRD, 1);

        // Configure line control: 8 bits, FIFO enabled
        self.write_reg(regs::LCR_H, lcr::WLEN_8 | lcr::FEN);

        // Enable UART, TX, and RX
        self.write_reg(regs::CR, cr::UARTEN | cr::TXE | cr::RXE);
    }

    /// Transmit a single byte.
    /// 
    /// Blocks until the transmit FIFO has space.
    pub fn putc(&self, c: u8) {
        // Wait until transmit FIFO is not full
        while self.read_reg(regs::FR) & flags::TXFF != 0 {
            core::hint::spin_loop();
        }
        
        // Write the character to the data register
        self.write_reg(regs::DR, c as u32);
    }

    /// Transmit a string.
    pub fn puts(&self, s: &str) {
        for byte in s.bytes() {
            // Convert newlines to CRLF for proper terminal output
            if byte == b'\n' {
                self.putc(b'\r');
            }
            self.putc(byte);
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.puts(s);
        Ok(())
    }
}

// =============================================================================
// Global UART Instance
// =============================================================================

/// Global UART instance, protected by a spinlock for thread-safety.
/// 
/// We use a static Mutex to allow multiple parts of the kernel to print
/// without stepping on each other's output.
static UART: Mutex<Uart> = Mutex::new(Uart::new(UART0_BASE));

/// Initialize the global UART.
pub fn init() {
    UART.lock().init();
}

/// Print a string to the UART.
pub fn puts(s: &str) {
    UART.lock().puts(s);
}

/// Print a formatted string to the UART.
pub fn _print(args: fmt::Arguments) {
    UART.lock().write_fmt(args).unwrap();
}

// =============================================================================
// Print Macros
// =============================================================================

/// Print to the kernel console.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::uart::_print(format_args!($($arg)*))
    };
}

/// Print to the kernel console with a newline.
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}

// =============================================================================
// Input Ring Buffer
// =============================================================================

struct RingBuffer {
    data: [u8; 128],
    head: usize,
    tail: usize,
}

impl RingBuffer {
    const fn new() -> Self {
        Self { data: [0; 128], head: 0, tail: 0 }
    }

    fn push(&mut self, byte: u8) {
        let next = (self.head + 1) % 128;
        if next != self.tail {
            self.data[self.head] = byte;
            self.head = next;
        }
    }


}

static RX_BUFFER: Mutex<RingBuffer> = Mutex::new(RingBuffer::new());

/// Handle UART Interrupt (Rx).
/// This is called from the exception handler.
pub fn handle_irq() {
    let uart = Uart::new(UART0_BASE);
    
    // Check Flags: RXFE (Receive FIFO Empty)
    // While RX FIFO is NOT empty...
    while uart.read_reg(regs::FR) & flags::RXFE == 0 {
        // Read byte
        let c = (uart.read_reg(regs::DR) & 0xFF) as u8;
        
        // Push to buffer
        RX_BUFFER.lock().push(c);
        
        // Echo back (for CLI feedback)
        if c == b'\r' {
            uart.putc(b'\r');
            uart.putc(b'\n');
        } else if c == 8 || c == 127 { // Backspace
            uart.putc(8);
            uart.putc(b' ');
            uart.putc(8);
        } else {
             uart.putc(c);
        }
    }
    
    // Clear RX Interrupt (RXIC) and Timeout (RTIC)
    // UARTICR (0x44) bit 4 (RXIC) and bit 6 (RTIC)
    uart.write_reg(0x44, (1 << 4) | (1 << 6));
}

/// Read a character from the serial port (non-blocking).
pub fn get_char() -> Option<u8> {
    // DEBUG: Polling Mode (Bypass Interrupts)
    let uart = Uart::new(UART0_BASE);
    if uart.read_reg(regs::FR) & flags::RXFE == 0 {
        let c = (uart.read_reg(regs::DR) & 0xFF) as u8;
        return Some(c);
    }
    None

    /*
    // Disable interrupts to prevent deadlock with IRQ handler
    crate::cpu::disable_interrupts();
    let result = RX_BUFFER.lock().pop();
    // Re-enable interrupts
    unsafe { crate::cpu::enable_interrupts(); }
    result
    */
}
