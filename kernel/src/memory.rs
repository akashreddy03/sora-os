use limine::memmap::{Entry, MEMMAP_USABLE};
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
};

const MAX_REGIONS: usize = 128;
pub struct BootFrameAllocator {
    usable_mem_regions: [MemMapRegion; MAX_REGIONS],
    length_mem_regions: usize,
    current_region: usize,
    next_phy_addr: PhysAddr,
}

#[derive(Default, Copy, Clone)]
pub struct MemMapRegion {
    base: u64,
    length: u64,
}

unsafe impl FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<x86_64::structures::paging::PhysFrame<Size4KiB>> {
        if self.next_phy_addr == PhysAddr::zero() {
            self.next_phy_addr = PhysAddr::new(self.usable_mem_regions[self.current_region].base);
        }

        if self.next_phy_addr
            >= PhysAddr::new(
                self.usable_mem_regions[self.current_region].base
                    + self.usable_mem_regions[self.current_region].length,
            )
        {
            self.current_region += 1;
        }

        if self.current_region >= self.length_mem_regions {
            return None;
        }

        let frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(self.next_phy_addr);
        self.next_phy_addr += 4096;
        Some(frame)
    }
}

impl BootFrameAllocator {
    pub fn init(memmap: &[&Entry]) -> BootFrameAllocator {
        let mut usable_mem_regions: [MemMapRegion; MAX_REGIONS] =
            [MemMapRegion { base: 0, length: 0 }; MAX_REGIONS];
        let mut count: usize = 0;

        for entry in memmap {
            if entry.type_ == MEMMAP_USABLE {
                usable_mem_regions[count] = MemMapRegion {
                    base: entry.base,
                    length: entry.length,
                };
                count += 1;
            }
        }

        BootFrameAllocator {
            usable_mem_regions,
            length_mem_regions: count,
            current_region: 0,
            next_phy_addr: PhysAddr::zero(),
        }
    }
}

/// # SAFETY
///
/// This function is unsafe because the caller must ensure the physical_memory_offset is correct.
/// Otherwise this function might break memory safety.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}
