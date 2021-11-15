use pe_loader::pe::{is_pe, relocate_pe_mem_with_per_sections, Sections};
use scroll::Pread;

fn fuzz_pe_loader(data: &[u8]) {
    {
        // is pe
        let mut status = is_pe(data);
        assert_eq!(status, true);
        let image_bytes = &mut [0u8; 10][..];
        status = is_pe(image_bytes);
        assert_eq!(status, false);

        let image_bytes = &mut [0u8; 0x55][..];
        status = is_pe(image_bytes);
        assert_eq!(status, false);

        image_bytes[0] = 0x4du8;
        image_bytes[1] = 0x5au8;
        status = is_pe(image_bytes);
        assert_eq!(status, false);

        image_bytes[0x3c] = 0x10;
        image_bytes[0x10] = 0x50;
        image_bytes[0x11] = 0x45;
        image_bytes[0x12] = 0x00;
        image_bytes[0x13] = 0x00;
        status = is_pe(image_bytes);
        assert_eq!(status, false);

        image_bytes[0x3c] = 0xAA;
        status = is_pe(image_bytes);
        assert_eq!(status, false);
    }
    {
        // sections
        let pe_image = data;
        let pe_header_offset = pe_image.pread::<u32>(0x3c).unwrap() as usize;
        let pe_region = &pe_image[pe_header_offset..];

        let num_sections = pe_region.pread::<u16>(6).unwrap() as usize;
        let optional_header_size = pe_region.pread::<u16>(20).unwrap() as usize;
        let optional_region = &pe_image[24 + pe_header_offset..];

        // check optional_hdr64_magic
        assert_eq!(optional_region.pread::<u16>(0).unwrap(), 0x20b);

        optional_region.pread::<u32>(16).unwrap();
        optional_region.pread::<u64>(24).unwrap();

        let sections_buffer = &pe_image[(24 + pe_header_offset + optional_header_size)..];

        let _total_header_size =
            (24 + pe_header_offset + optional_header_size + num_sections * 40) as usize;

        Sections::parse(sections_buffer, num_sections as usize).unwrap();
    }
    {
        // relocate_pe_mem_with_per_sections
        let mut loaded_buffer = vec![0u8; 0x200000];
        let pe_image = data;

        relocate_pe_mem_with_per_sections(pe_image, loaded_buffer.as_mut_slice(), |_| ());
    }
}

fn main() {
    #[cfg(not(feature = "fuzz"))]
    {
        let mut args = std::env::args().skip(1);
        if let Some(arg) = args.next() {
            println!("{}", arg);
            let data = std::fs::read(arg).expect("read crash file fail");
            fuzz_pe_loader(data.as_slice());
        }
    }
    #[cfg(feature = "fuzz")]
    afl::fuzz!(|data: &[u8]| {
        fuzz_pe_loader(data);
    });
}
