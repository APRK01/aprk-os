use aprk_arch_arm64::{print, println, uart};
use alloc::string::String;
use alloc::vec::Vec;

pub extern "C" fn run() {
    println!("\n[shell] Welcome to APRK OS Shell!");
    println!("Type 'help' for commands.");
    print!("> ");
    
    let mut buffer = String::new();
    
    unsafe { aprk_arch_arm64::cpu::enable_interrupts(); }

    loop {
        if let Some(c) = uart::get_char() {
            if c == b'\n' || c == b'\r' {
                println!(); // Newline
                execute_command(buffer.trim());
                buffer.clear();
                print!("> ");
            } else if c == 8 || c == 127 {
                // Backspace
                if !buffer.is_empty() {
                     buffer.pop();
                }
            } else {
                buffer.push(c as char);
            }
        } else {
            // No input? Spin loop or yield?
            // Just spin loop for now.
             for _ in 0..10_000 { unsafe { core::arch::asm!("nop") } }
        }
    }
}

fn execute_command(cmd: &str) {
    match cmd {
        "help" => {
             println!("Available commands:");
             println!("  help     - Show this menu");
             println!("  clear    - Clear screen");
             println!("  whoami   - Print user info");
             println!("  uname    - Print system info");
        },
        "clear" => print!("\x1b[2J\x1b[1;1H"),
        "whoami" => println!("root (KERNEL_GOD_MODE)"),
        "uname" => println!("APRK OS v0.0.1 aarch64"),
        "" => {},
        _ => println!("Unknown command: '{}'", cmd),
    }
}
