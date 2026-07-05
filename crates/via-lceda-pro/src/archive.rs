use std::io::{self, Write};

use crate::constants::{ZIP_DOS_DATE, ZIP_DOS_TIME};

pub(crate) struct ZipArchive {
    files: Vec<ZipFile>,
}

struct ZipFile {
    name: String,
    contents: Vec<u8>,
}

impl ZipArchive {
    pub(crate) fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub(crate) fn add_file(&mut self, name: String, contents: Vec<u8>) {
        self.files.push(ZipFile { name, contents });
    }

    pub(crate) fn finish(self) -> io::Result<Vec<u8>> {
        let mut out = Vec::new();
        let mut central_directory = Vec::new();
        let file_count = checked_u16(self.files.len())?;

        for file in self.files {
            let offset = checked_u32(out.len())?;
            let name = file.name.as_bytes();
            let contents = &file.contents;
            let crc = crc32(contents);
            let size = checked_u32(contents.len())?;
            let name_len = checked_u16(name.len())?;

            put_u32(&mut out, 0x0403_4b50);
            put_u16(&mut out, 20);
            put_u16(&mut out, 0);
            put_u16(&mut out, 0);
            put_u16(&mut out, ZIP_DOS_TIME);
            put_u16(&mut out, ZIP_DOS_DATE);
            put_u32(&mut out, crc);
            put_u32(&mut out, size);
            put_u32(&mut out, size);
            put_u16(&mut out, name_len);
            put_u16(&mut out, 0);
            out.write_all(name)?;
            out.write_all(contents)?;

            put_u32(&mut central_directory, 0x0201_4b50);
            put_u16(&mut central_directory, 20);
            put_u16(&mut central_directory, 20);
            put_u16(&mut central_directory, 0);
            put_u16(&mut central_directory, 0);
            put_u16(&mut central_directory, ZIP_DOS_TIME);
            put_u16(&mut central_directory, ZIP_DOS_DATE);
            put_u32(&mut central_directory, crc);
            put_u32(&mut central_directory, size);
            put_u32(&mut central_directory, size);
            put_u16(&mut central_directory, name_len);
            put_u16(&mut central_directory, 0);
            put_u16(&mut central_directory, 0);
            put_u16(&mut central_directory, 0);
            put_u16(&mut central_directory, 0);
            put_u32(&mut central_directory, 0);
            put_u32(&mut central_directory, offset);
            central_directory.write_all(name)?;
        }

        let central_offset = checked_u32(out.len())?;
        let central_size = checked_u32(central_directory.len())?;
        out.write_all(&central_directory)?;
        put_u32(&mut out, 0x0605_4b50);
        put_u16(&mut out, 0);
        put_u16(&mut out, 0);
        put_u16(&mut out, file_count);
        put_u16(&mut out, file_count);
        put_u32(&mut out, central_size);
        put_u32(&mut out, central_offset);
        put_u16(&mut out, 0);

        Ok(out)
    }
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn checked_u16(value: usize) -> io::Result<u16> {
    value
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "zip field exceeds u16"))
}

fn checked_u32(value: usize) -> io::Result<u32> {
    value
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "zip field exceeds u32"))
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}
