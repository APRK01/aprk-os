extern "C" {
    pub fn context_switch(prev_sp: *mut u64, next_sp: u64);
}
