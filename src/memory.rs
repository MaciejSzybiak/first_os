use x86_64::VirtAddr;
use x86_64::PhysAddr;
use x86_64::structures::paging::{PhysFrame, Size4KiB, FrameAllocator, PageTable, OffsetPageTable};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

// physical memory should be mapped to virtual memory at the passed memory offset
unsafe fn active_level_4_table(physical_mem_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (l4_table_frame, _) = Cr3::read();

    let physical_address = l4_table_frame.start_address();
    let virtual_address = physical_mem_offset + physical_address.as_u64();
    let page_table_pointer: *mut PageTable = virtual_address.as_mut_ptr();

    &mut *page_table_pointer
}

// initializes an OffsetPageTable
pub unsafe fn init(physical_mem_offset: VirtAddr) -> OffsetPageTable<'static> {
    let l4_table = active_level_4_table(physical_mem_offset);
    OffsetPageTable::new(l4_table, physical_mem_offset)
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap, // from bootloader
    next: usize,
}

impl BootInfoFrameAllocator {
    // creates a frame allocator for the passed memory map
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    // returns an iterator for usable frames in the memory map
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // find usable regions
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        
        //map regions to address ranges
        let address_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());

        let frame_addresses= address_ranges.flat_map(|r| r.step_by(4096)); //4kb page size
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
