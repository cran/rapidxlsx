## ----include = FALSE----------------------------------------------------------
knitr::opts_chunk$set(
  collapse = TRUE,
  comment = "#>"
)

## ----setup--------------------------------------------------------------------
library(rapidxlsx)

df <- data.frame(
  id = 101:105,
  name = c("Alice", "Bob", "Charlie", "David", "Eva"),
  score = c(95.5, 88.0, 92.3, 76.4, 89.1),
  active = c(TRUE, FALSE, TRUE, TRUE, FALSE),
  group = factor(c("A", "B", "A", "C", "B")),
  stringsAsFactors = FALSE
)

tmp <- tempfile(fileext = ".xlsx")
write_xlsx(df, tmp, sheet_name = "Participants")

## ----piping-------------------------------------------------------------------
out_path <- df |>
  subset(active == TRUE) |>
  write_xlsx(tempfile(fileext = ".xlsx"), sheet_name = "ActiveUsers")

file.exists(out_path)

## ----na-handling--------------------------------------------------------------
data_with_na <- data.frame(
  x = c(1.0, NA, 3.5),
  y = c("valid", "text", NA),
  stringsAsFactors = FALSE
)

tmp_na <- tempfile(fileext = ".xlsx")
write_xlsx(data_with_na, tmp_na)

## ----compression--------------------------------------------------------------
tmp_comp <- tempfile(fileext = ".xlsx")
write_xlsx(df, tmp_comp, compression_level = 6L)

## ----cleanup, include=FALSE---------------------------------------------------
unlink(tmp)
unlink(out_path)
unlink(tmp_na)
unlink(tmp_comp)

