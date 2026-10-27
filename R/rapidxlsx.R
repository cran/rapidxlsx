write_xlsx <- function(data, path, sheet_name = "Sheet1", compression_level = 1L) {
  if (!is.data.frame(data)) {
    data <- as.data.frame(data)
  }

  path <- as.character(path)[1]
  sheet_name <- as.character(sheet_name)[1]
  compression_level <- as.integer(compression_level)[1]

  .Call(c_write_xlsx, path, sheet_name, data, compression_level)
  invisible(path)
}
