# 한 줄 주석
add <- function(a, b = 2) {
  a + b
}

x <- c(1, 2.5, 3L)
name <- "Don Editor"
if (length(x) > 2 && !is.null(name)) {
  for (i in seq_along(x)) {
    cat("값:", x[i], "\n")
  }
}
df <- data.frame(id = 1:3, label = c("가", "나", "다"))
result <- sapply(df$id, function(v) v^2)
print(result)
