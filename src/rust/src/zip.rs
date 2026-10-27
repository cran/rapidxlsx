use std::io::{self, Write};

pub struct ZipEntry {
    pub name: String,
    pub data: Vec<u8>,
    pub uncompressed_size: u64,
    pub crc32: u32,
    pub is_deflated: bool,
}

pub struct ZipPackage {
    entries: Vec<ZipEntry>,
}

impl ZipPackage {
    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(8),
        }
    }

    pub fn add_stored(&mut self, name: &str, data: Vec<u8>) {
        let crc = crc32fast::hash(&data);
        let len = data.len() as u64;
        self.entries.push(ZipEntry {
            name: name.to_string(),
            data,
            uncompressed_size: len,
            crc32: crc,
            is_deflated: false,
        });
    }

    pub fn add_deflated(
        &mut self,
        name: &str,
        compressed_data: Vec<u8>,
        uncompressed_size: u64,
        crc32: u32,
    ) {
        self.entries.push(ZipEntry {
            name: name.to_string(),
            data: compressed_data,
            uncompressed_size,
            crc32,
            is_deflated: true,
        });
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        struct StoredMeta {
            name: Vec<u8>,
            method: u16,
            crc32: u32,
            compressed_size: u64,
            uncompressed_size: u64,
            local_header_offset: u64,
        }

        let mut metas = Vec::with_capacity(self.entries.len());
        let mut current_offset: u64 = 0;

        for entry in &self.entries {
            let name_bytes = entry.name.as_bytes();
            let method: u16 = if entry.is_deflated { 8 } else { 0 };
            let comp_len = entry.data.len() as u64;
            let uncomp_len = entry.uncompressed_size;
            let offset = current_offset;

            let needs_zip64 = comp_len >= 0xFFFF_FFFF
                || uncomp_len >= 0xFFFF_FFFF
                || offset >= 0xFFFF_FFFF;

            let mut extra_field = Vec::new();
            if needs_zip64 {
                extra_field.extend_from_slice(&0x0001u16.to_le_bytes());
                extra_field.extend_from_slice(&16u16.to_le_bytes());
                extra_field.extend_from_slice(&uncomp_len.to_le_bytes());
                extra_field.extend_from_slice(&comp_len.to_le_bytes());
            }

            let version_needed: u16 = if needs_zip64 { 45 } else { 20 };
            let comp_32: u32 = if comp_len >= 0xFFFF_FFFF {
                0xFFFF_FFFF
            } else {
                comp_len as u32
            };
            let uncomp_32: u32 = if uncomp_len >= 0xFFFF_FFFF {
                0xFFFF_FFFF
            } else {
                uncomp_len as u32
            };

            writer.write_all(&0x04034b50u32.to_le_bytes())?;
            writer.write_all(&version_needed.to_le_bytes())?;
            writer.write_all(&0u16.to_le_bytes())?;
            writer.write_all(&method.to_le_bytes())?;
            writer.write_all(&0u16.to_le_bytes())?;
            writer.write_all(&0x0021u16.to_le_bytes())?;
            writer.write_all(&entry.crc32.to_le_bytes())?;
            writer.write_all(&comp_32.to_le_bytes())?;
            writer.write_all(&uncomp_32.to_le_bytes())?;
            writer.write_all(&(name_bytes.len() as u16).to_le_bytes())?;
            writer.write_all(&(extra_field.len() as u16).to_le_bytes())?;
            writer.write_all(name_bytes)?;
            if !extra_field.is_empty() {
                writer.write_all(&extra_field)?;
            }
            writer.write_all(&entry.data)?;

            let header_len = 30 + name_bytes.len() as u64 + extra_field.len() as u64;
            current_offset += header_len + comp_len;

            metas.push(StoredMeta {
                name: name_bytes.to_vec(),
                method,
                crc32: entry.crc32,
                compressed_size: comp_len,
                uncompressed_size: uncomp_len,
                local_header_offset: offset,
            });
        }

        let central_dir_start = current_offset;

        for meta in &metas {
            let needs_zip64 = meta.compressed_size >= 0xFFFF_FFFF
                || meta.uncompressed_size >= 0xFFFF_FFFF
                || meta.local_header_offset >= 0xFFFF_FFFF;

            let mut extra_field = Vec::new();
            if needs_zip64 {
                extra_field.extend_from_slice(&0x0001u16.to_le_bytes());
                let mut size: u16 = 0;
                if meta.uncompressed_size >= 0xFFFF_FFFF {
                    size += 8;
                }
                if meta.compressed_size >= 0xFFFF_FFFF {
                    size += 8;
                }
                if meta.local_header_offset >= 0xFFFF_FFFF {
                    size += 8;
                }
                extra_field.extend_from_slice(&size.to_le_bytes());
                if meta.uncompressed_size >= 0xFFFF_FFFF {
                    extra_field.extend_from_slice(&meta.uncompressed_size.to_le_bytes());
                }
                if meta.compressed_size >= 0xFFFF_FFFF {
                    extra_field.extend_from_slice(&meta.compressed_size.to_le_bytes());
                }
                if meta.local_header_offset >= 0xFFFF_FFFF {
                    extra_field.extend_from_slice(&meta.local_header_offset.to_le_bytes());
                }
            }

            let version_needed: u16 = if needs_zip64 { 45 } else { 20 };
            let comp_32: u32 = if meta.compressed_size >= 0xFFFF_FFFF {
                0xFFFF_FFFF
            } else {
                meta.compressed_size as u32
            };
            let uncomp_32: u32 = if meta.uncompressed_size >= 0xFFFF_FFFF {
                0xFFFF_FFFF
            } else {
                meta.uncompressed_size as u32
            };
            let offset_32: u32 = if meta.local_header_offset >= 0xFFFF_FFFF {
                0xFFFF_FFFF
            } else {
                meta.local_header_offset as u32
            };

            writer.write_all(&0x02014b50u32.to_le_bytes())?;
            writer.write_all(&0x0314u16.to_le_bytes())?;
            writer.write_all(&version_needed.to_le_bytes())?;
            writer.write_all(&0u16.to_le_bytes())?;
            writer.write_all(&meta.method.to_le_bytes())?;
            writer.write_all(&0u16.to_le_bytes())?;
            writer.write_all(&0x0021u16.to_le_bytes())?;
            writer.write_all(&meta.crc32.to_le_bytes())?;
            writer.write_all(&comp_32.to_le_bytes())?;
            writer.write_all(&uncomp_32.to_le_bytes())?;
            writer.write_all(&(meta.name.len() as u16).to_le_bytes())?;
            writer.write_all(&(extra_field.len() as u16).to_le_bytes())?;
            writer.write_all(&0u16.to_le_bytes())?;
            writer.write_all(&0u16.to_le_bytes())?;
            writer.write_all(&0u16.to_le_bytes())?;
            writer.write_all(&0u32.to_le_bytes())?;
            writer.write_all(&offset_32.to_le_bytes())?;
            writer.write_all(&meta.name)?;
            if !extra_field.is_empty() {
                writer.write_all(&extra_field)?;
            }

            current_offset += 46 + meta.name.len() as u64 + extra_field.len() as u64;
        }

        let central_dir_size = current_offset - central_dir_start;
        let total_entries = metas.len() as u64;

        let needs_zip64_eocd = total_entries >= 0xFFFF
            || central_dir_size >= 0xFFFF_FFFF
            || central_dir_start >= 0xFFFF_FFFF;

        if needs_zip64_eocd {
            let zip64_eocd_offset = current_offset;

            writer.write_all(&0x06064b50u32.to_le_bytes())?;
            writer.write_all(&44u64.to_le_bytes())?;
            writer.write_all(&45u16.to_le_bytes())?;
            writer.write_all(&45u16.to_le_bytes())?;
            writer.write_all(&0u32.to_le_bytes())?;
            writer.write_all(&0u32.to_le_bytes())?;
            writer.write_all(&total_entries.to_le_bytes())?;
            writer.write_all(&total_entries.to_le_bytes())?;
            writer.write_all(&central_dir_size.to_le_bytes())?;
            writer.write_all(&central_dir_start.to_le_bytes())?;

            writer.write_all(&0x07064b50u32.to_le_bytes())?;
            writer.write_all(&0u32.to_le_bytes())?;
            writer.write_all(&zip64_eocd_offset.to_le_bytes())?;
            writer.write_all(&1u32.to_le_bytes())?;
        }

        let entries_16 = if total_entries >= 0xFFFF {
            0xFFFF
        } else {
            total_entries as u16
        };
        let cd_size_32 = if central_dir_size >= 0xFFFF_FFFF {
            0xFFFF_FFFF
        } else {
            central_dir_size as u32
        };
        let cd_start_32 = if central_dir_start >= 0xFFFF_FFFF {
            0xFFFF_FFFF
        } else {
            central_dir_start as u32
        };

        writer.write_all(&0x06054b50u32.to_le_bytes())?;
        writer.write_all(&0u16.to_le_bytes())?;
        writer.write_all(&0u16.to_le_bytes())?;
        writer.write_all(&entries_16.to_le_bytes())?;
        writer.write_all(&entries_16.to_le_bytes())?;
        writer.write_all(&cd_size_32.to_le_bytes())?;
        writer.write_all(&cd_start_32.to_le_bytes())?;
        writer.write_all(&0u16.to_le_bytes())?;

        writer.flush()?;
        Ok(())
    }
}
