// // src/audio/decoder.rs

// use std::collections::HashMap;
// use std::fs;
// use std::io::Cursor;
// use std::sync::{LazyLock, RwLock};
// use symphonia::core::audio::SampleBuffer;
// use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
// use symphonia::core::formats::{FormatOptions, FormatReader};
// use symphonia::core::io::MediaSourceStream;
// use symphonia::core::meta::MetadataOptions;
// use symphonia::core::probe::Hint;
// use symphonia::default::{get_codecs, get_probe};

// pub static DECODED_AUDIO_CACHE: LazyLock<RwLock<HashMap<String, Vec<f32>>>> =
//     LazyLock::new(|| RwLock::new(HashMap::new()));

// pub fn get_format_reader(file_path: &str) -> Box<dyn FormatReader> {
//     let file_bytes = fs::read(file_path).expect("failed to read file into memory");
//     let cursor = Cursor::new(file_bytes);
//     let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

//     let mut hint = Hint::new();
//     if let Some(ext) = file_path.split('.').last() {
//         hint.with_extension(ext);
//     }

//     let fmt_opts: FormatOptions = Default::default();
//     let meta_opts: MetadataOptions = Default::default();

//     let probed = get_probe()
//         .format(&hint, mss, &fmt_opts, &meta_opts)
//         .expect("unsupported format");

//     return probed.format;
// }

// pub fn decode_audio_sample(file_path: &str) -> Vec<f32> {
//     let mut format_reader = get_format_reader(file_path);

//     let track = format_reader
//         .default_track()
//         .expect("no default track found");

//     let dec_opts: DecoderOptions = Default::default();
//     let mut decoder = get_codecs()
//         .make(&track.codec_params, &dec_opts)
//         .expect("unsupported codec");

//     loop {
//         let packet = match format_reader.next_packet() {
//             Ok(packet) => packet,
//             Err(_) => break,
//         };

//         if packet.track_id() != track.id {
//             continue;
//         }

//         match decoder.decode(&packet) {
//             Ok(decoded) => {
//                 let mut temp_buffer =
//                     SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());

//                 temp_buffer.copy_interleaved_ref(decoded);
//                 // samples.extend_from_slice(temp_buffer.samples());
//             }
//             Err(_) => continue,
//         }
//     }

//     return vec![];
// }
