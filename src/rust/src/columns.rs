use crate::xml::{write_bool_cell, write_float_cell, write_int_cell, write_string_cell};

pub enum ColumnData<'a> {
    Int64(Vec<i64>),
    Int64Slice(&'a [i64]),
    Int64Opt(Vec<Option<i64>>),
    Int32(Vec<i32>),
    Int32Slice(&'a [i32]),
    Int32RLogical(&'a [i32]),
    Int32RNullable(&'a [i32]),
    Float64(Vec<f64>),
    Float64Slice(&'a [f64]),
    Float64Opt(Vec<Option<f64>>),
    Float32(Vec<f32>),
    Float32Slice(&'a [f32]),
    Bool(Vec<bool>),
    BoolSlice(&'a [bool]),
    BoolOpt(Vec<Option<bool>>),
    StringDense(Vec<String>),
    StringSlice(&'a [&'a str]),
    StringOpt(Vec<Option<String>>),
    Empty(usize),
}

impl<'a> ColumnData<'a> {
    #[inline(always)]
    pub fn len(&self) -> usize {
        match self {
            Self::Int64(v) => v.len(),
            Self::Int64Slice(v) => v.len(),
            Self::Int64Opt(v) => v.len(),
            Self::Int32(v) => v.len(),
            Self::Int32Slice(v) => v.len(),
            Self::Int32RLogical(v) => v.len(),
            Self::Int32RNullable(v) => v.len(),
            Self::Float64(v) => v.len(),
            Self::Float64Slice(v) => v.len(),
            Self::Float64Opt(v) => v.len(),
            Self::Float32(v) => v.len(),
            Self::Float32Slice(v) => v.len(),
            Self::Bool(v) => v.len(),
            Self::BoolSlice(v) => v.len(),
            Self::BoolOpt(v) => v.len(),
            Self::StringDense(v) => v.len(),
            Self::StringSlice(v) => v.len(),
            Self::StringOpt(v) => v.len(),
            Self::Empty(len) => *len,
        }
    }

    #[inline(always)]
    pub fn write_cell(
        &self,
        row_idx: usize,
        excel_row: u32,
        col_idx: usize,
        buf: &mut Vec<u8>,
        itoa_buf: &mut itoa::Buffer,
        ryu_buf: &mut ryu::Buffer,
    ) {
        match self {
            Self::Int64(v) => {
                if let Some(&val) = v.get(row_idx) {
                    write_int_cell(buf, col_idx, excel_row, val, itoa_buf);
                }
            }
            Self::Int64Slice(v) => {
                if let Some(&val) = v.get(row_idx) {
                    write_int_cell(buf, col_idx, excel_row, val, itoa_buf);
                }
            }
            Self::Int64Opt(v) => {
                if let Some(Some(val)) = v.get(row_idx) {
                    write_int_cell(buf, col_idx, excel_row, *val, itoa_buf);
                }
            }
            Self::Int32(v) => {
                if let Some(&val) = v.get(row_idx) {
                    write_int_cell(buf, col_idx, excel_row, val as i64, itoa_buf);
                }
            }
            Self::Int32Slice(v) => {
                if let Some(&val) = v.get(row_idx) {
                    write_int_cell(buf, col_idx, excel_row, val as i64, itoa_buf);
                }
            }
            Self::Int32RNullable(v) => {
                if let Some(&val) = v.get(row_idx) {
                    if val != i32::MIN {
                        write_int_cell(buf, col_idx, excel_row, val as i64, itoa_buf);
                    }
                }
            }
            Self::Int32RLogical(v) => {
                if let Some(&val) = v.get(row_idx) {
                    if val != i32::MIN {
                        write_bool_cell(buf, col_idx, excel_row, val != 0, itoa_buf);
                    }
                }
            }
            Self::Float64(v) => {
                if let Some(&val) = v.get(row_idx) {
                    if !val.is_nan() {
                        write_float_cell(buf, col_idx, excel_row, val, itoa_buf, ryu_buf);
                    }
                }
            }
            Self::Float64Slice(v) => {
                if let Some(&val) = v.get(row_idx) {
                    if !val.is_nan() {
                        write_float_cell(buf, col_idx, excel_row, val, itoa_buf, ryu_buf);
                    }
                }
            }
            Self::Float64Opt(v) => {
                if let Some(Some(val)) = v.get(row_idx) {
                    if !val.is_nan() {
                        write_float_cell(buf, col_idx, excel_row, *val, itoa_buf, ryu_buf);
                    }
                }
            }
            Self::Float32(v) => {
                if let Some(&val) = v.get(row_idx) {
                    if !val.is_nan() {
                        write_float_cell(buf, col_idx, excel_row, val as f64, itoa_buf, ryu_buf);
                    }
                }
            }
            Self::Float32Slice(v) => {
                if let Some(&val) = v.get(row_idx) {
                    if !val.is_nan() {
                        write_float_cell(buf, col_idx, excel_row, val as f64, itoa_buf, ryu_buf);
                    }
                }
            }
            Self::Bool(v) => {
                if let Some(&val) = v.get(row_idx) {
                    write_bool_cell(buf, col_idx, excel_row, val, itoa_buf);
                }
            }
            Self::BoolSlice(v) => {
                if let Some(&val) = v.get(row_idx) {
                    write_bool_cell(buf, col_idx, excel_row, val, itoa_buf);
                }
            }
            Self::BoolOpt(v) => {
                if let Some(Some(val)) = v.get(row_idx) {
                    write_bool_cell(buf, col_idx, excel_row, *val, itoa_buf);
                }
            }
            Self::StringDense(v) => {
                if let Some(s) = v.get(row_idx) {
                    write_string_cell(buf, col_idx, excel_row, s, itoa_buf);
                }
            }
            Self::StringSlice(v) => {
                if let Some(&s) = v.get(row_idx) {
                    write_string_cell(buf, col_idx, excel_row, s, itoa_buf);
                }
            }
            Self::StringOpt(v) => {
                if let Some(Some(s)) = v.get(row_idx) {
                    write_string_cell(buf, col_idx, excel_row, s, itoa_buf);
                }
            }
            Self::Empty(_) => {}
        }
    }
}

unsafe impl<'a> Send for ColumnData<'a> {}
unsafe impl<'a> Sync for ColumnData<'a> {}
