use x86_64::structures::paging::{Page, Size4KiB, Mapper, FrameAllocator, MapperAllSizes, PageTable, PhysFrame, MappedPageTable};
use x86_64::{PhysAddr, VirtAddr};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

/// Initialize a new MappedPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: u64, memory_map: &'static MemoryMap) -> (impl MapperAllSizes, PhysicalFrameAllocator) {
    let level_4_table = active_level_4_table(physical_memory_offset);
    let phys_to_virt = move |frame: PhysFrame| -> *mut PageTable {
        let phys = frame.start_address().as_u64();
        let virt = VirtAddr::new(phys + physical_memory_offset);
        virt.as_mut_ptr()
    };

    let frame_allocator = PhysicalFrameAllocator::init(memory_map, physical_memory_offset);

    (MappedPageTable::new(level_4_table, phys_to_virt), frame_allocator)
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: u64)
    -> &'static mut PageTable
{
    use x86_64::{registers::control::Cr3};

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = VirtAddr::new(phys.as_u64() + physical_memory_offset);
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

/// Creates an example mapping for the given page to frame `0xb8000`.
pub fn create_example_mapping(
    page: Page,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

struct FreeFrame {
    next_frame: Option<*mut FreeFrame>,
    previous_frame: Option<*mut FreeFrame>
}

pub struct PhysicalFrameAllocator {
    physical_memory_offset: u64,
    first_free: Option<*mut FreeFrame>,
    free_frame_count: u64,
    used_frame_count: u64
}

impl PhysicalFrameAllocator {
    pub fn init(memory_map: &'static MemoryMap, physical_memory_offset: u64) -> PhysicalFrameAllocator {
        let frames = PhysicalFrameAllocator::usable_frames(memory_map);

        let mut frame_count: u64 = 0;
        let mut previous_frame: Option<*mut FreeFrame> = None;
        let mut first_free: Option<*mut FreeFrame> = None;
        for frame in frames {
            let free_frame = (frame.start_address().as_u64() + physical_memory_offset) as *mut FreeFrame;
            unsafe { (*free_frame).previous_frame = previous_frame; }
            match previous_frame {
                Some(p) => unsafe { (*p).next_frame = Some(free_frame) },
                None => first_free = Some(free_frame)
            };
            previous_frame = Some(free_frame);
            frame_count += 1;
        }
        match previous_frame {
            Some(p) => unsafe { (*p).next_frame = None },
            None => ()
        }

        PhysicalFrameAllocator { physical_memory_offset: physical_memory_offset, first_free: first_free, free_frame_count: frame_count, used_frame_count: 0 }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(memory_map: &'static MemoryMap) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses
            .map(|addr|PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysicalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let mut first_free = self.first_free.unwrap();
        let mut next_frame: *mut FreeFrame;
        unsafe { next_frame = (*first_free).next_frame.unwrap(); }

        self.first_free = Some(next_frame);
        unsafe { (*next_frame).previous_frame = None; }
        self.free_frame_count -= 1;
        self.used_frame_count += 1;

        Some(PhysFrame::from_start_address(PhysAddr::new(first_free as u64 - self.physical_memory_offset)).unwrap())
    }
}