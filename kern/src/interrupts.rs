use x86_64::{
    instructions::tables::sidt,
    structures::idt::InterruptDescriptorTable
};

pub fn see_whats_there() -> &'static InterruptDescriptorTable {
    unsafe { sidt().base.as_ptr::<InterruptDescriptorTable>().as_ref().unwrap() }
}
