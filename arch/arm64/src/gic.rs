// =============================================================================
// APRK OS - ARM Generic Interrupt Controller (GICv2)
// =============================================================================
// Driver for the GICv2 interrupt controller used by QEMU's virt machine.
//
// The GIC consists of:
// - Distributor: Prioritizes and routes interrupts to CPUs.
// - CPU Interface: Handles interrupt masking and acknowledgement for a specific CPU.
// =============================================================================

use core::ptr;

// QEMU virt machine GICv2 base addresses
const GICD_BASE: usize = 0x0800_0000;
const GICC_BASE: usize = 0x0801_0000;

// Distributor Registers
const GICD_CTLR: usize = 0x000;       // Control Register
const GICD_ISENABLER: usize = 0x100;  // Interrupt Set-Enable Registers

// CPU Interface Registers
const GICC_CTLR: usize = 0x0000;      // Control Register
const GICC_PMR: usize = 0x0004;       // Priority Mask Register
const GICC_IAR: usize = 0x000C;       // Interrupt Acknowledge Register
const GICC_EOIR: usize = 0x0010;      // End of Interrupt Register

pub struct Gic;

impl Gic {
    /// Initialize the GIC (Distributor and CPU Interface).
    ///
    /// # Safety
    /// Must be called only once on boot.
    pub unsafe fn init() {
        // ---------------------------------------------------------------------
        // 1. Distributor Initialization
        // ---------------------------------------------------------------------
        // Enable the distributor
        write_gicd(GICD_CTLR, 1);

        // Enable the timer interrupt (ID 27 for virtual timer)
        let timer_irq = 27;
        let reg_offset = (timer_irq / 32) * 4;
        let bit = 1 << (timer_irq % 32);
        
        // Read-Modify-Write
        let mut current_enable = read_gicd(GICD_ISENABLER + reg_offset);
        current_enable |= bit;
        write_gicd(GICD_ISENABLER + reg_offset, current_enable);

        // Enable UART Interrupt (ID 33)
        // ID 33 is likely in ISENABLER1 (32-63)
        let uart_irq = 33;
        let reg_offset_u = (uart_irq / 32) * 4;
        let bit_u = 1 << (uart_irq % 32);
        
        let mut current_enable_u = read_gicd(GICD_ISENABLER + reg_offset_u);
        current_enable_u |= bit_u;
        write_gicd(GICD_ISENABLER + reg_offset_u, current_enable_u);

        // ---------------------------------------------------------------------
        // 2. CPU Interface Initialization
        // ---------------------------------------------------------------------
        // Set Priority Mask to 0xFF (allow all interrupts)
        write_gicc(GICC_PMR, 0xFF);

        // Enable the CPU interface
        write_gicc(GICC_CTLR, 1);
    }

    /// Acknowledge the currently pending interrupt.
    /// Returns the Interrupt ID (IAR value).
    pub fn acknowledge() -> u32 {
        unsafe { read_gicc(GICC_IAR) }
    }

    /// Signal End Of Interrupt (EOI).
    /// Tells the GIC we are done handling this interrupt.
    pub fn end_interrupt(id: u32) {
        unsafe { write_gicc(GICC_EOIR, id) }
    }
}

// Helper to read distributor register
unsafe fn read_gicd(offset: usize) -> u32 {
    ptr::read_volatile((GICD_BASE + offset) as *const u32)
}

// Helper to write distributor register
unsafe fn write_gicd(offset: usize, value: u32) {
    ptr::write_volatile((GICD_BASE + offset) as *mut u32, value)
}

// Helper to read CPU interface register
unsafe fn read_gicc(offset: usize) -> u32 {
    ptr::read_volatile((GICC_BASE + offset) as *const u32)
}

// Helper to write CPU interface register
unsafe fn write_gicc(offset: usize, value: u32) {
    ptr::write_volatile((GICC_BASE + offset) as *mut u32, value)
}
