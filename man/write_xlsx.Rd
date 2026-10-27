\name{write_xlsx}
\alias{write_xlsx}
\title{Write Data Frame to XLSX File}
\usage{
write_xlsx(data, path, sheet_name = "Sheet1", compression_level = 1L)
}
\arguments{
\item{data}{A data.frame or object coercible to a data.frame.}

\item{path}{Path to output XLSX file.}

\item{sheet_name}{Name of worksheet (default: "Sheet1").}

\item{compression_level}{Deflate compression level (1 to 9, default: 1).}
}
\value{
Invisibly returns the path to the written file.
}
\description{
Writes tabular data to an Excel XLSX workbook using a multi-threaded Rust engine with parallel Deflate compression.
}
\examples{
df <- data.frame(
  id = 1:5,
  name = c("Alice", "Bob", "Charlie", "David", "Eva"),
  score = c(95.5, 82.0, 91.5, 78.0, 88.5),
  active = c(TRUE, FALSE, TRUE, TRUE, FALSE)
)
tmp <- tempfile(fileext = ".xlsx")
write_xlsx(df, tmp)
}
