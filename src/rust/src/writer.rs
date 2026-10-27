use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;

use flate2::{Compress, Compression, FlushCompress};
use rayon::prelude::*;

use crate::columns::ColumnData;
use crate::xml::{append_escaped_xml, col_name, write_string_cell};
use crate::zip::ZipPackage;

static CONTENT_TYPES_XML: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
</Types>"#;

static ROOT_RELS_XML: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#;

static WORKBOOK_RELS_XML: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;

static STYLES_XML: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>
  <fills count="2"><fill><patternFill/></fill><fill><patternFill patternType="gray125"/></fill></fills>
  <borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>
  <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
  <cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/></cellXfs>
  <cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>
</styleSheet>"#;

static SHEET_SUFFIX: &[u8] = br#"</sheetData><pageMargins left="0.7" right="0.7" top="0.75" bottom="0.75" header="0.3" footer="0.3"/></worksheet>"#;

pub fn write_xlsx_file<P: AsRef<Path>>(
    path: P,
    sheet_name: &str,
    headers: &[String],
    columns: &[ColumnData],
    compression_level: u32,
) -> io::Result<()> {
    let num_cols = headers.len().max(columns.len());
    let num_rows = columns.iter().map(|c| c.len()).max().unwrap_or(0);

    if num_rows > 1_048_575 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("row count {} exceeds Excel limit of 1,048,576", num_rows + 1),
        ));
    }
    if num_cols > 16_384 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("column count {} exceeds Excel limit of 16,384", num_cols),
        ));
    }

    let mut itoa_main = itoa::Buffer::new();
    let has_headers = !headers.is_empty();

    let last_col_idx = if num_cols > 0 { num_cols - 1 } else { 0 };
    let last_col = col_name(last_col_idx);
    let total_excel_rows = if has_headers { num_rows + 1 } else { num_rows };
    let total_excel_rows = total_excel_rows.max(1) as u32;

    let mut prefix_buf = Vec::with_capacity(512);
    prefix_buf.extend_from_slice(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
    prefix_buf.extend_from_slice(b"<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">");
    prefix_buf.extend_from_slice(b"<dimension ref=\"A1:");
    prefix_buf.extend_from_slice(last_col.as_bytes());
    prefix_buf.extend_from_slice(itoa_main.format(total_excel_rows).as_bytes());
    prefix_buf.extend_from_slice(b"\"/>");
    prefix_buf.extend_from_slice(b"<sheetViews><sheetView tabSelected=\"1\" workbookViewId=\"0\"/></sheetViews>");
    prefix_buf.extend_from_slice(b"<sheetFormatPr defaultRowHeight=\"15\"/>");
    prefix_buf.extend_from_slice(b"<sheetData>");

    if has_headers {
        prefix_buf.extend_from_slice(b"<row r=\"1\">");
        for (c, h) in headers.iter().enumerate() {
            write_string_cell(&mut prefix_buf, c, 1, h, &mut itoa_main);
        }
        prefix_buf.extend_from_slice(b"</row>");
    }

    let num_threads = rayon::current_num_threads();
    let num_chunks = (num_threads * 2).max(1);
    let chunk_size = (num_rows + num_chunks - 1) / num_chunks;

    let active_chunks = if num_rows > 0 {
        ((num_rows + chunk_size - 1) / chunk_size).max(1)
    } else {
        0
    };

    struct ChunkResult {
        compressed_bytes: Vec<u8>,
        uncompressed_len: usize,
        hasher: crc32fast::Hasher,
    }

    let comp_level_val = compression_level.clamp(1, 9);

    let compressed_chunks: Vec<ChunkResult> = if active_chunks > 0 {
        (0..active_chunks)
            .into_par_iter()
            .map(|chunk_idx| {
                let start_row = chunk_idx * chunk_size;
                let end_row = ((chunk_idx + 1) * chunk_size).min(num_rows);

                let is_first = chunk_idx == 0;
                let is_last = chunk_idx == active_chunks - 1;

                let row_count = end_row - start_row;
                let estimated_cap = (row_count * num_cols * 50).max(4096)
                    + (if is_first { prefix_buf.len() } else { 0 })
                    + (if is_last { SHEET_SUFFIX.len() } else { 0 });

                let mut raw_buf = Vec::with_capacity(estimated_cap);
                if is_first {
                    raw_buf.extend_from_slice(&prefix_buf);
                }

                let mut itoa_buf = itoa::Buffer::new();
                let mut ryu_buf = ryu::Buffer::new();

                for r in start_row..end_row {
                    let excel_row = if has_headers {
                        (r + 2) as u32
                    } else {
                        (r + 1) as u32
                    };

                    raw_buf.extend_from_slice(b"<row r=\"");
                    raw_buf.extend_from_slice(itoa_buf.format(excel_row).as_bytes());
                    raw_buf.extend_from_slice(b"\">");

                    for (c, col) in columns.iter().enumerate() {
                        col.write_cell(r, excel_row, c, &mut raw_buf, &mut itoa_buf, &mut ryu_buf);
                    }

                    raw_buf.extend_from_slice(b"</row>");
                }

                if is_last {
                    raw_buf.extend_from_slice(SHEET_SUFFIX);
                }

                let uncompressed_len = raw_buf.len();
                let mut hasher = crc32fast::Hasher::new();
                hasher.update(&raw_buf);

                let mut compressor = Compress::new(Compression::new(comp_level_val), false);
                let mut comp_buf = Vec::with_capacity((uncompressed_len / 4).max(1024));

                let flush_mode = if is_last {
                    FlushCompress::Finish
                } else {
                    FlushCompress::Sync
                };

                let mut in_pos = 0;
                loop {
                    let before_in = compressor.total_in() as usize;
                    let before_out = compressor.total_out() as usize;

                    let avail_out = comp_buf.capacity() - comp_buf.len();
                    if avail_out < 4096 {
                        comp_buf.reserve(uncompressed_len / 4 + 4096);
                    }

                    let spare = comp_buf.spare_capacity_mut();
                    let dest = unsafe {
                        std::slice::from_raw_parts_mut(spare.as_mut_ptr() as *mut u8, spare.len())
                    };

                    let res = compressor
                        .compress(&raw_buf[in_pos..], dest, flush_mode)
                        .expect("compress failed");

                    let consumed = (compressor.total_in() as usize) - before_in;
                    let written = (compressor.total_out() as usize) - before_out;

                    in_pos += consumed;
                    unsafe {
                        let new_len = comp_buf.len() + written;
                        comp_buf.set_len(new_len);
                    }

                    if is_last {
                        if res == flate2::Status::StreamEnd {
                            break;
                        }
                    } else if in_pos == raw_buf.len() && res == flate2::Status::Ok {
                        break;
                    }
                }

                ChunkResult {
                    compressed_bytes: comp_buf,
                    uncompressed_len,
                    hasher,
                }
            })
            .collect()
    } else {
        let mut raw = Vec::new();
        raw.extend_from_slice(&prefix_buf);
        raw.extend_from_slice(SHEET_SUFFIX);
        let uncomp_len = raw.len();
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&raw);

        let mut compressor = Compress::new(Compression::new(comp_level_val), false);
        let mut comp_buf = Vec::with_capacity(1024);
        let mut in_pos = 0;
        loop {
            let before_in = compressor.total_in() as usize;
            let before_out = compressor.total_out() as usize;
            if comp_buf.capacity() - comp_buf.len() < 1024 {
                comp_buf.reserve(1024);
            }
            let spare = comp_buf.spare_capacity_mut();
            let dest = unsafe {
                std::slice::from_raw_parts_mut(spare.as_mut_ptr() as *mut u8, spare.len())
            };
            let res = compressor
                .compress(&raw[in_pos..], dest, FlushCompress::Finish)
                .expect("compress empty sheet failed");
            let consumed = (compressor.total_in() as usize) - before_in;
            let written = (compressor.total_out() as usize) - before_out;
            in_pos += consumed;
            unsafe {
                let new_len = comp_buf.len() + written;
                comp_buf.set_len(new_len);
            }
            if res == flate2::Status::StreamEnd {
                break;
            }
        }

        vec![ChunkResult {
            compressed_bytes: comp_buf,
            uncompressed_len: uncomp_len,
            hasher,
        }]
    };

    let total_uncompressed_len: u64 = compressed_chunks
        .iter()
        .map(|c| c.uncompressed_len as u64)
        .sum();
    let total_comp_bytes: usize = compressed_chunks
        .iter()
        .map(|c| c.compressed_bytes.len())
        .sum();

    let mut final_deflate_data = Vec::with_capacity(total_comp_bytes);
    for chunk in &compressed_chunks {
        final_deflate_data.extend_from_slice(&chunk.compressed_bytes);
    }

    let mut combined_hasher = compressed_chunks[0].hasher.clone();
    for i in 1..compressed_chunks.len() {
        combined_hasher.combine(&compressed_chunks[i].hasher);
    }
    let combined_crc = combined_hasher.finalize();

    let mut workbook_xml = Vec::with_capacity(256);
    workbook_xml.extend_from_slice(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name=""#);
    append_escaped_xml(&mut workbook_xml, sheet_name);
    workbook_xml.extend_from_slice(br#"" sheetId="1" r:id="rId1"/>
  </sheets>
</workbook>"#);

    let mut zip = ZipPackage::new();
    zip.add_stored("[Content_Types].xml", CONTENT_TYPES_XML.to_vec());
    zip.add_stored("_rels/.rels", ROOT_RELS_XML.to_vec());
    zip.add_stored("xl/_rels/workbook.xml.rels", WORKBOOK_RELS_XML.to_vec());
    zip.add_stored("xl/workbook.xml", workbook_xml);
    zip.add_stored("xl/styles.xml", STYLES_XML.to_vec());
    zip.add_deflated(
        "xl/worksheets/sheet1.xml",
        final_deflate_data,
        total_uncompressed_len,
        combined_crc,
    );

    let file = File::create(path)?;
    let mut writer = BufWriter::with_capacity(1024 * 1024, file);
    zip.write_to(&mut writer)?;

    Ok(())
}
