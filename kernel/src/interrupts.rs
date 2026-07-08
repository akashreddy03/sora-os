use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use spin::{LazyLock, Mutex};
use crate::{serial_println, serial_print};
use crate::gdt;
use pic8259::ChainedPics;

use pc_keyboard::{HandleControl, PS2Keyboard, ScancodeSet1, layouts};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
}

pub static PICS: Mutex<ChainedPics> = {
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) })
};

pub static IDT: LazyLock<InterruptDescriptorTable> = LazyLock::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);    
    unsafe { 
        idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX); 
    }
    idt.page_fault.set_handler_fn(page_fault_handler);
    // idt.segment_not_present.set_handler_fn(segment_not_present_handler);
    idt.general_protection_fault.set_handler_fn(segment_not_present_handler);
    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
    idt
});

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = x86_64::instructions::port::Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    static KEYBOARD: Mutex<PS2Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(PS2Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore));

    let mut keyboard = KEYBOARD.lock();
    
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                pc_keyboard::DecodedKey::Unicode(character) => serial_print!("{}", character), 
                pc_keyboard::DecodedKey::RawKey(_key) => ()
            }
        }
    }

    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8()); }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8()); }
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    serial_println!("EXCEPTION: PAGE FAULT, ERROR CODE: \n{:#?} \n{:#?}", error_code, stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
    serial_println!("EXCEPTION: SEGMENT NOT PRESENT\n{:#?}", stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    serial_println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    crate::hlt_loop();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
} 