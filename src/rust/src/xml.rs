use std::sync::LazyLock;

static COL_NAMES: LazyLock<Vec<Box<str>>> = LazyLock::new(|| {
    let mut cols = Vec::with_capacity(16384);
    for i in 0..16384 {
        cols.push(calc_col_name(i).into_boxed_str());
    }
    cols
});

fn calc_col_name(mut idx: usize) -> String {
    let mut s = String::with_capacity(4);
    let mut temp = [0u8; 4];
    let mut len = 0;
    loop {
        temp[len] = b'A' + (idx % 26) as u8;
        len += 1;
        if idx < 26 {
            break;
        }
        idx = (idx / 26) - 1;
    }
    for i in 0..len {
        s.push(temp[len - 1 - i] as char);
    }
    s
}

#[inline(always)]
pub fn col_name(idx: usize) -> &'static str {
    if idx < COL_NAMES.len() {
        &COL_NAMES[idx]
    } else {
        "A"
    }
}

#[inline(always)]
pub fn append_cell_ref(buf: &mut Vec<u8>, col_idx: usize, row_idx: u32, itoa_buf: &mut itoa::Buffer) {
    buf.extend_from_slice(col_name(col_idx).as_bytes());
    buf.extend_from_slice(itoa_buf.format(row_idx).as_bytes());
}

#[inline(always)]
pub fn append_escaped_xml(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    let mut last = 0;

    for (i, &b) in bytes.iter().enumerate() {
        let replacement: Option<&[u8]> = match b {
            b'&' => Some(b"&amp;"),
            b'<' => Some(b"&lt;"),
            b'>' => Some(b"&gt;"),
            b'"' => Some(b"&quot;"),
            b'\'' => Some(b"&apos;"),
            0..=8 | 11 | 12 | 14..=31 => Some(b""),
            _ => None,
        };

        if let Some(rep) = replacement {
            if i > last {
                buf.extend_from_slice(&bytes[last..i]);
            }
            buf.extend_from_slice(rep);
            last = i + 1;
        }
    }

    if last < bytes.len() {
        buf.extend_from_slice(&bytes[last..]);
    }
}

#[inline(always)]
pub fn write_string_cell(
    buf: &mut Vec<u8>,
    col_idx: usize,
    row_idx: u32,
    val: &str,
    itoa_buf: &mut itoa::Buffer,
) {
    buf.extend_from_slice(b"<c r=\"");
    append_cell_ref(buf, col_idx, row_idx, itoa_buf);
    let bytes = val.as_bytes();
    let preserve = if !bytes.is_empty() {
        bytes[0] == b' ' || bytes[bytes.len() - 1] == b' '
    } else {
        false
    };
    if preserve {
        buf.extend_from_slice(b"\" t=\"inlineStr\"><is><t xml:space=\"preserve\">");
    } else {
        buf.extend_from_slice(b"\" t=\"inlineStr\"><is><t>");
    }
    append_escaped_xml(buf, val);
    buf.extend_from_slice(b"</t></is></c>");
}

#[inline(always)]
pub fn write_int_cell(
    buf: &mut Vec<u8>,
    col_idx: usize,
    row_idx: u32,
    val: i64,
    itoa_buf: &mut itoa::Buffer,
) {
    buf.extend_from_slice(b"<c r=\"");
    append_cell_ref(buf, col_idx, row_idx, itoa_buf);
    buf.extend_from_slice(b"\"><v>");
    buf.extend_from_slice(itoa_buf.format(val).as_bytes());
    buf.extend_from_slice(b"</v></c>");
}

#[inline(always)]
pub fn write_float_cell(
    buf: &mut Vec<u8>,
    col_idx: usize,
    row_idx: u32,
    val: f64,
    itoa_buf: &mut itoa::Buffer,
    ryu_buf: &mut ryu::Buffer,
) {
    buf.extend_from_slice(b"<c r=\"");
    append_cell_ref(buf, col_idx, row_idx, itoa_buf);
    if val.is_nan() {
        buf.extend_from_slice(b"\" t=\"e\"><v>#NUM!</v></c>");
    } else if val.is_infinite() {
        buf.extend_from_slice(b"\" t=\"e\"><v>#DIV/0!</v></c>");
    } else {
        buf.extend_from_slice(b"\"><v>");
        buf.extend_from_slice(ryu_buf.format(val).as_bytes());
        buf.extend_from_slice(b"</v></c>");
    }
}

#[inline(always)]
pub fn write_bool_cell(
    buf: &mut Vec<u8>,
    col_idx: usize,
    row_idx: u32,
    val: bool,
    itoa_buf: &mut itoa::Buffer,
) {
    buf.extend_from_slice(b"<c r=\"");
    append_cell_ref(buf, col_idx, row_idx, itoa_buf);
    if val {
        buf.extend_from_slice(b"\" t=\"b\"><v>1</v></c>");
    } else {
        buf.extend_from_slice(b"\" t=\"b\"><v>0</v></c>");
    }
}
