#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]
use core::panic::PanicInfo;
use limine::BaseRevision;
use limine::request::FramebufferRequest;
mod serial;
use core::fmt::Write;

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[test_case]
fn simple_test() {
    serial_println!("Simple Test...");
    assert_eq!(1, 1);
    serial_println!("[ok]");
}

static FONT: &[u8] = include_bytes!("../fonts/Lat2-Terminus16.psfu");

#[used]
#[link_name = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new(); 

#[used]
#[link_name = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

fn draw_pixel(framebuffer: *mut u8, x: usize, y:usize, pitch: usize, color: u32) {
    let offset = y * pitch + x * 4;
    unsafe {
        *(framebuffer.add(offset) as *mut u32) = color;
    }
}

struct Psf2Header {
    magic: u32,
    version: u32,
    headersize: u32,
    flags: u32,
    glyphs: u32,
    bytes_per_glyph: u32,
    height: u32,
    width: u32
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset+1],
        data[offset+2],
        data[offset+3]
    ])
}

fn read_psf2_header() -> Psf2Header {
    Psf2Header {
        magic: read_u32(FONT, 0),
        version: read_u32(FONT, 4),
        headersize: read_u32(FONT, 2 * 4),
        flags: read_u32(FONT, 3 * 4),
        glyphs: read_u32(FONT, 4 * 4),
        bytes_per_glyph: read_u32(FONT, 5 * 4),
        height: read_u32(FONT, 6 * 4),
        width: read_u32(FONT, 7 * 4), 
    }
}

fn draw_char(framebuffer: *mut (), x: usize, y:usize, pitch: usize, c: usize) {
    let header = read_psf2_header();
    let glyph_start = header.headersize as usize + c * header.bytes_per_glyph as usize;

    for row in 0..header.height as usize {
        let bits = FONT[glyph_start + row];
        for col in 0..header.width as usize {
            if bits & (1 << 7 - col) != 0 {
                draw_pixel(framebuffer as *mut u8, x + col, y + row, pitch, 0xFFFFFFFF);
            }
        }
    }

}

fn draw_string(framebuffer: *mut (), x:usize, y:usize, pitch: usize, string: &str) {
    for (i, c) in string.chars().enumerate() {
        draw_char(framebuffer, x + 8 * i, y, pitch, c as usize);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    assert!(BaseRevision::is_supported(&BASE_REVISION));

    serial_println!("hello World");

    #[cfg(test)]
    test_main();

    serial_println!("end of program");

    /* 
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().first() {
            
            draw_string(framebuffer.address(), 10 as usize,10 as usize, framebuffer.pitch as usize, "Hello World!");

        }
    }*/

    loop {}
}

#[panic_handler]
fn panic(_panicinfo: &PanicInfo) -> ! {
    loop {}
}


