library(rapidxlsx)

test_file <- tempfile(fileext = ".xlsx")
exdir <- tempfile()

df <- data.frame(
  id = c(101L, 102L, NA),
  name = c("Alice", NA, "Charlie \u00e9\u00e0\u4e16\u754c"),
  score = c(95.5, 88.0, NA),
  active = c(TRUE, FALSE, NA),
  category = factor(c("High", "Low", NA)),
  stringsAsFactors = FALSE
)

res_path <- write_xlsx(df, test_file, sheet_name = "TestData")
stopifnot(identical(res_path, test_file))
stopifnot(file.exists(test_file))
stopifnot(file.size(test_file) > 0)

utils::unzip(test_file, files = "xl/worksheets/sheet1.xml", exdir = exdir)
sheet_xml <- paste(readLines(file.path(exdir, "xl/worksheets/sheet1.xml"), warn = FALSE), collapse = " ")

stopifnot(grepl("Alice", sheet_xml, fixed = TRUE))
stopifnot(grepl("Charlie \u00e9\u00e0\u4e16\u754c", sheet_xml, fixed = TRUE))
stopifnot(grepl("95.5", sheet_xml, fixed = TRUE))
stopifnot(grepl("88", sheet_xml, fixed = TRUE))
stopifnot(grepl("High", sheet_xml, fixed = TRUE))
stopifnot(grepl("Low", sheet_xml, fixed = TRUE))
stopifnot(grepl("<v>1</v>", sheet_xml, fixed = TRUE))
stopifnot(grepl("<v>0</v>", sheet_xml, fixed = TRUE))

single_file <- tempfile(fileext = ".xlsx")
res_single <- write_xlsx(data.frame(x = 1.0, y = "solo"), single_file)
stopifnot(identical(res_single, single_file))
stopifnot(file.exists(single_file))

large_file <- tempfile(fileext = ".xlsx")
n_test <- 50000L
df_large <- data.frame(
  id = seq_len(n_test),
  val = seq(0, 100, length.out = n_test),
  flag = rep(c(TRUE, FALSE), length.out = n_test)
)
write_xlsx(df_large, large_file, sheet_name = "LargeData")
stopifnot(file.exists(large_file))
stopifnot(file.size(large_file) > 100000)

unlink(test_file)
unlink(single_file)
unlink(large_file)
unlink(exdir, recursive = TRUE)

cat("All R unit and output value verification tests passed.\n")
