#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::{hint::black_box, panic::PanicInfo};
use limine::request::{HhdmRequest, MemmapRequest};
use sora_os::{
    exit_qemu, gdt::DOUBLE_FAULT_IST_INDEX, hlt_loop, serial_print, serial_println,
    test_panic_handler,
};
use spin::LazyLock;
use x86_64::{
    VirtAddr,
    structures::{
        idt::{InterruptDescriptorTable, InterruptStackFrame},
        paging::{FrameAllocator, Mapper, Page, PageTableFlags},
    },
};

#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();

static IDT_TEST: LazyLock<InterruptDescriptorTable> = LazyLock::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    unsafe {
        idt.double_fault
            .set_handler_fn(test_double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX)
    };
    idt
});

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    let hhdm_address_response = HHDM_REQUEST
        .response()
        .expect("HHDM Address is not passed by the bootloader.");
    let memmap = MEMMAP_REQUEST
        .response()
        .expect("Memmap is not passed by the bootloader");
    let physical_memory_offset = VirtAddr::new(hhdm_address_response.offset);

    let mut mapper = unsafe { sora_os::memory::init(physical_memory_offset) };

    let mut frame_allocator =
        unsafe { sora_os::memory::BootFrameAllocator::init(memmap.entries()) };

    // Initalize all the components
    sora_os::init();

    // Load Test IDT
    IDT_TEST.load();

    // Create a new stack memory
    let stack_start: VirtAddr = VirtAddr::new(0x_3333_3333_0000);
    let stack_size: usize = 4096 * 5;
    let stack_start_page = Page::containing_address(stack_start);
    let stack_end_page = Page::containing_address(stack_start + stack_size as u64 - 1);

    // Map pages to physical frames
    for page in Page::range_inclusive(stack_start_page, stack_end_page) {
        let frame = frame_allocator.allocate_frame().unwrap();
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper
                .map_to(page, frame, flags, &mut frame_allocator)
                .expect("Mapping failed")
                .flush();
        };
    }

    // Create a guard page by setting page table flags to 0
    unsafe {
        mapper
            .update_flags(stack_start_page, PageTableFlags::empty())
            .expect("Updating Flags Failed")
            .flush();
    }

    // Change RSP to point to the new stack memory with guard page
    unsafe {
        core::arch::asm!(
            "mov rsp, {}",
            in(reg) (stack_start + stack_size as u64 - 1).as_u64(),
        );
    }

    // Excecute the infintely recursing function
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(sora_os::QemuExitStatus::Success);
    hlt_loop();
}

// An infinite recursive function
#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    black_box(());
}
