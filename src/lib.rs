#![warn(clippy::all)]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub mod __memory {
    use std::{mem, alloc as std_alloc};

    #[no_mangle]
    pub unsafe extern "C" fn alloc(size: usize) -> *mut u8 {
        let layout = std_alloc::Layout::from_size_align(size, mem::align_of::<usize>()).unwrap();
        std_alloc::alloc(layout)
    }

    #[no_mangle]
    pub unsafe extern "C" fn dealloc(ptr: *mut u8, size: usize) {
        if size == 0 { return }
        let layout = std_alloc::Layout::from_size_align_unchecked(size, mem::align_of::<usize>());
        std_alloc::dealloc(ptr, layout)
    }
}

pub mod __vorbis {
    use std::{slice, boxed::Box, io::Cursor};
    use lewton::inside_ogg::OggStreamReader;
    use hound::{WavWriter, WavSpec, SampleFormat};

    #[no_mangle]
    pub unsafe extern "C" fn decode_vorbis(ptr: *const u8, size: usize, p_wav_size: *mut usize) -> *mut u8 {
        let input = slice::from_raw_parts(ptr, size);
        let mut output = Cursor::new(Vec::new());

        let mut reader = OggStreamReader::new(Cursor::new(input)).unwrap();
        let mut writer = WavWriter::new(&mut output, WavSpec {
            channels: u16::from(reader.ident_hdr.audio_channels),
            sample_rate: reader.ident_hdr.audio_sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        }).unwrap();

        while let Some(samples) = reader.read_dec_packet_itl().unwrap() {
            for sample in samples {
                writer.write_sample(sample).unwrap();
            }
        };
        writer.finalize().unwrap();

        let output_box = output.into_inner().into_boxed_slice();
        *p_wav_size = output_box.len();
        Box::into_raw(output_box) as *mut u8
    }
}
