suppressPackageStartupMessages({
  library(rvest)
  library(xml2)
  library(dplyr)
  library(stringr)
  library(lubridate)
  library(ggplot2)
})

options(width = 140)

primary_url <- "https://chamspage.blogspot.com/2025/01/2025-baltimore-city-homicide-list.html"
feed_url <- "https://chamspage.blogspot.com/feeds/posts/default?q=Baltimore%20City%20Homicide&max-results=100"
output_dir <- Sys.getenv("OUTPUT_DIR", unset = "output")
dir.create(output_dir, showWarnings = FALSE, recursive = TRUE)

`%||%` <- function(x, y) {
  if (is.null(x) || length(x) == 0 || all(is.na(x))) y else x
}

clean_text <- function(x) {
  x <- x %||% NA_character_
  x <- as.character(x)
  x <- str_replace_all(x, "\u00a0", " ")
  x <- str_replace_all(x, "[\r\n\t]+", " ")
  x <- str_squish(x)
  x[x == ""] <- NA_character_
  x
}

normalize_url <- function(x) {
  x <- str_replace(x, "^http://", "https://")
  str_replace(x, "[?#].*$", "")
}

extract_year <- function(x) {
  suppressWarnings(as.integer(str_extract(x, "20[0-9]{2}")))
}

discover_pages <- function() {
  fallback <- data.frame(
    list_year = 2015:2026,
    title = c(
      "2015 Baltimore City Homicides/Murders - List and Map",
      "2016 Baltimore City Homicides List and Map",
      "2017 Baltimore City Homicides - List and Map",
      "2018 Baltimore City Homicides - List and Map",
      "2019 Baltimore City Homicides - List",
      "2020 Baltimore City Homicides - List",
      "2021 Baltimore City Homicides - List",
      "2022 Baltimore City Homicides List",
      "2023 Baltimore City Homicides List",
      "2024 Baltimore City Homicide List",
      "2025 Baltimore City Homicide List",
      "2026 Baltimore City Homicide List"
    ),
    url = c(
      "https://chamspage.blogspot.com/2015/11/2015-baltimore-city-homicidesmurders.html",
      "https://chamspage.blogspot.com/2016/03/2016-baltimore-city-homicides-list-and.html",
      "https://chamspage.blogspot.com/2017/01/2017-baltimore-city-homicides-list-and.html",
      "https://chamspage.blogspot.com/2018/01/2018-baltimore-city-homicides-list-and.html",
      "https://chamspage.blogspot.com/2019/01/2019-baltimore-city-homicides-list.html",
      "https://chamspage.blogspot.com/2020/01/2020-baltimore-city-homicides-list.html",
      "https://chamspage.blogspot.com/2021/01/2021-baltimore-city-homicides-list.html",
      "https://chamspage.blogspot.com/2022/01/2022-baltimore-city-homicides-list.html",
      "https://chamspage.blogspot.com/2023/01/2023-baltimore-city-homicides-list.html",
      "https://chamspage.blogspot.com/2024/01/2024-baltimore-city-homicide-list.html",
      primary_url,
      "https://chamspage.blogspot.com/2026/01/2026-baltimore-city-homicide-list.html"
    ),
    stringsAsFactors = FALSE
  )

  feed <- tryCatch(xml2::read_xml(feed_url), error = function(e) NULL)
  if (is.null(feed)) {
    return(fallback)
  }

  entries <- xml_find_all(feed, "//*[local-name()='entry']")
  if (!length(entries)) {
    return(fallback)
  }

  titles <- clean_text(xml_text(xml_find_first(entries, "./*[local-name()='title']")))
  links <- xml_attr(xml_find_first(entries, "./*[local-name()='link'][@rel='alternate']"), "href")
  pages <- data.frame(
    title = titles,
    url = normalize_url(links),
    stringsAsFactors = FALSE
  )

  pages <- pages %>%
    mutate(
      list_year = extract_year(title),
      title_lower = str_to_lower(title)
    ) %>%
    filter(
      !is.na(list_year),
      list_year >= 2015,
      str_detect(title_lower, "baltimore city"),
      str_detect(title_lower, "homicide"),
      str_detect(title_lower, "list"),
      !str_detect(title_lower, "county")
    ) %>%
    select(list_year, title, url)

  pages <- bind_rows(pages, fallback) %>%
    mutate(url = normalize_url(url)) %>%
    distinct(list_year, .keep_all = TRUE) %>%
    arrange(list_year)

  pages
}

row_cells <- function(row) {
  xml_find_all(row, "./th|./td")
}

table_score <- function(tbl) {
  rows <- xml_find_all(tbl, "./thead/tr|./tbody/tr|./tr")
  if (!length(rows)) {
    return(0L)
  }

  sample_rows <- rows[seq_len(min(3, length(rows)))]
  header_text <- str_to_lower(clean_text(paste(xml_text(xml_find_all(sample_rows, "./th|./td")), collapse = " ")))
  sum(str_detect(header_text, c("date died", "name", "age", "address", "notes", "case closed")))
}

find_homicide_table <- function(page) {
  tables <- rvest::html_elements(page, "table")
  if (!length(tables)) {
    stop("No HTML tables found on page.")
  }

  scores <- vapply(tables, table_score, integer(1))
  best <- which.max(scores)
  if (scores[[best]] < 4) {
    stop("Could not identify a homicide table from table headers.")
  }
  tables[[best]]
}

standard_record <- function(values) {
  n <- length(values)
  out <- rep(NA_character_, 9)

  if (n == 9) {
    out <- values
  } else if (n > 9) {
    out[1:5] <- values[1:5]
    out[6] <- clean_text(paste(values[6:(n - 3)], collapse = " "))
    out[7:9] <- values[(n - 2):n]
  } else if (n >= 6) {
    out[seq_len(n)] <- values
  } else {
    return(NULL)
  }

  out
}

parse_homicide_table <- function(url, title, list_year) {
  page <- rvest::read_html(url)
  tbl <- find_homicide_table(page)
  rows <- xml_find_all(tbl, "./thead/tr|./tbody/tr|./tr")

  header_idx <- which(vapply(rows, function(row) {
    txt <- str_to_lower(clean_text(paste(xml_text(row_cells(row)), collapse = " ")))
    str_detect(txt, "date died") && str_detect(txt, "name") && str_detect(txt, "age")
  }, logical(1)))[1]

  if (is.na(header_idx)) {
    stop("Could not find the homicide table header row.")
  }

  data_rows <- rows[(header_idx + 1):length(rows)]
  records <- lapply(data_rows, function(row) {
    cells <- row_cells(row)
    values <- clean_text(xml_text(cells))
    values <- standard_record(values)
    if (is.null(values)) {
      return(NULL)
    }

    number <- values[[1]]
    if (is.na(number) || !str_detect(number, "^(\\d{1,4}[A-Za-z]?|[Xx]+)$")) {
      return(NULL)
    }

    name_url <- NA_character_
    if (length(cells) >= 3) {
      href <- xml_attr(xml_find_first(cells[[3]], ".//a[@href]"), "href")
      name_url <- normalize_url(href %||% NA_character_)
    }

    data.frame(
      list_year = list_year,
      source_title = title,
      source_url = url,
      number = values[[1]],
      date_died_raw = values[[2]],
      victim_name = values[[3]],
      age_raw = values[[4]],
      address_block_found = values[[5]],
      notes = values[[6]],
      victim_has_no_violent_criminal_history = values[[7]],
      surveillance_camera_at_intersection = values[[8]],
      case_status = values[[9]],
      name_url = name_url,
      stringsAsFactors = FALSE
    )
  })

  bind_rows(records)
}

parse_death_date <- function(x) {
  x <- clean_text(x)
  first_full_date <- str_extract(x, "\\b\\d{1,2}/\\d{1,2}/\\d{2,4}\\b")
  first_month_year <- str_extract(x, "\\b\\d{1,2}/\\d{4}\\b")
  first_year <- str_extract(x, "\\b20\\d{2}\\b")
  candidate <- ifelse(!is.na(first_full_date), first_full_date,
    ifelse(!is.na(first_month_year), paste0("01/", first_month_year),
      ifelse(!is.na(first_year), paste0("01/01/", first_year), NA_character_)
    )
  )

  parsed <- suppressWarnings(lubridate::mdy(candidate, quiet = TRUE))
  as.Date(parsed)
}

infer_method <- function(notes) {
  notes_lower <- str_to_lower(notes %||% "")
  case_when(
    str_detect(notes_lower, "shoot|gunshot|\\bshot\\b") ~ "Shooting",
    str_detect(notes_lower, "stab|cutting|knife") ~ "Stabbing/cutting",
    str_detect(notes_lower, "blunt|trauma|beaten|assault") ~ "Blunt force/trauma",
    str_detect(notes_lower, "vehicle|hit[- ]and[- ]run|struck|collision|vehicular") ~ "Vehicle",
    str_detect(notes_lower, "fire|burn") ~ "Fire/burning",
    TRUE ~ "Other/unknown"
  )
}

parse_camera_count <- function(x) {
  count <- suppressWarnings(as.integer(str_extract(x %||% "", "\\d+")))
  count
}

pages <- discover_pages()

cat("Source pages selected from Chamspage/Blogger feed:\n")
print(pages, row.names = FALSE)
cat("\n")

raw_records <- bind_rows(lapply(seq_len(nrow(pages)), function(i) {
  row <- pages[i, ]
  message(sprintf("Scraping %s: %s", row$list_year, row$url))
  tryCatch(
    parse_homicide_table(row$url, row$title, row$list_year),
    error = function(e) {
      warning(sprintf("Skipping %s (%s): %s", row$list_year, row$url, conditionMessage(e)))
      data.frame()
    }
  )
}))

if (!nrow(raw_records)) {
  stop("No homicide records could be scraped.")
}

homicides <- raw_records %>%
  mutate(
    number_int = suppressWarnings(as.integer(str_extract(number, "\\d+"))),
    date_died = parse_death_date(date_died_raw),
    death_month = lubridate::month(date_died, label = TRUE, abbr = TRUE),
    death_wday = lubridate::wday(date_died, label = TRUE, abbr = TRUE),
    age = suppressWarnings(as.integer(str_extract(age_raw, "\\d{1,3}"))),
    age = ifelse(!is.na(age) & age >= 0 & age <= 110, age, NA_integer_),
    inferred_method = infer_method(notes),
    camera_count = parse_camera_count(surveillance_camera_at_intersection),
    camera_present = case_when(
      str_detect(str_to_lower(surveillance_camera_at_intersection %||% ""), "no camera|none") ~ FALSE,
      !is.na(camera_count) & camera_count > 0 ~ TRUE,
      str_detect(str_to_lower(surveillance_camera_at_intersection %||% ""), "camera|yes") ~ TRUE,
      TRUE ~ NA
    ),
    case_closed = case_when(
      str_detect(str_to_lower(case_status %||% ""), "closed") ~ TRUE,
      is.na(case_status) ~ NA,
      TRUE ~ FALSE
    )
  )

clean_csv <- file.path(output_dir, "baltimore_homicides_cleaned.csv")
write.csv(homicides, clean_csv, row.names = FALSE, na = "")

analysis_data <- homicides %>%
  filter(!is.na(number_int), !is.na(age), age >= 0, age <= 100)

if (!nrow(analysis_data)) {
  stop("No numbered homicide records had usable victim ages.")
}

bin_width <- 5
max_age <- ceiling(max(analysis_data$age, na.rm = TRUE) / 10) * 10
age_breaks <- seq(0, max(100, max_age), by = bin_width)
analysis_data <- analysis_data %>%
  mutate(
    age_bin_lower = floor(age / bin_width) * bin_width,
    age_bin = sprintf("%02d-%02d", age_bin_lower, age_bin_lower + bin_width - 1)
  )

age_levels <- sprintf("%02d-%02d", age_breaks[-length(age_breaks)], age_breaks[-1] - 1)
analysis_data$age_bin <- factor(analysis_data$age_bin, levels = age_levels)

hist_long <- analysis_data %>%
  count(age_bin, inferred_method, name = "victims") %>%
  filter(!is.na(age_bin))

hist_matrix <- xtabs(victims ~ age_bin + inferred_method, data = hist_long)
hist_table <- data.frame(
  age_bin = rownames(hist_matrix),
  total_victims = as.integer(rowSums(hist_matrix)),
  as.data.frame.matrix(hist_matrix),
  check.names = FALSE,
  row.names = NULL
) %>%
  filter(total_victims > 0)

peak <- hist_table[which.max(hist_table$total_victims), c("age_bin", "total_victims")]
year_range <- sprintf("%s-%s", min(analysis_data$list_year), max(analysis_data$list_year))

method_palette <- c(
  "Shooting" = "#2C7FB8",
  "Stabbing/cutting" = "#F28E2B",
  "Blunt force/trauma" = "#7B6FD6",
  "Vehicle" = "#59A14F",
  "Fire/burning" = "#E15759",
  "Other/unknown" = "#707070"
)

histogram <- ggplot(analysis_data, aes(x = age, fill = inferred_method)) +
  geom_histogram(binwidth = bin_width, boundary = 0, color = "white", linewidth = 0.25) +
  scale_x_continuous(
    breaks = seq(0, max(100, max_age), by = 10),
    limits = c(0, max(100, max_age))
  ) +
  scale_fill_manual(values = method_palette, drop = FALSE) +
  labs(
    title = paste("Baltimore City homicide victim ages,", year_range),
    subtitle = sprintf(
      "%s numbered victims with usable ages scraped from %s annual Chamspage tables; colors use method inferred from notes.",
      nrow(analysis_data),
      n_distinct(analysis_data$list_year)
    ),
    x = "Victim age",
    y = "Victims",
    fill = "Inferred method"
  ) +
  annotate(
    "label",
    x = max(100, max_age) * 0.76,
    y = peak$total_victims * 0.88,
    label = sprintf("Peak age bin: %s\n%s victims", peak$age_bin, peak$total_victims),
    size = 3.4,
    linewidth = 0.2,
    fill = "white"
  ) +
  coord_cartesian(clip = "off") +
  theme_minimal(base_size = 12) +
  theme(
    plot.title = element_text(face = "bold"),
    panel.grid.minor = element_blank(),
    legend.position = "bottom"
  )

plot_path <- file.path(output_dir, "baltimore_homicide_age_histogram.png")
ggsave(plot_path, histogram, width = 11, height = 7, dpi = 160)

cat("Scraped rows by list year:\n")
print(
  homicides %>%
    count(list_year, name = "rows_scraped") %>%
    arrange(list_year),
  row.names = FALSE
)
cat("\n")

cat("Cleaning summary:\n")
cat(sprintf("- Raw rows scraped: %s\n", nrow(homicides)))
cat(sprintf("- Numbered rows with usable ages included in histogram: %s\n", nrow(analysis_data)))
cat(sprintf("- Rows excluded from histogram because they were non-numbered or age was missing/out of range: %s\n", nrow(homicides) - nrow(analysis_data)))
cat(sprintf("- Cleaned CSV: %s\n", clean_csv))
cat(sprintf("- Histogram image: %s\n\n", plot_path))

cat("Histogram data: victim age bins by inferred method\n")
print(hist_table, row.names = FALSE)
cat("\n")

cat("Overall inferred method mix for histogram records:\n")
print(
  analysis_data %>%
    count(inferred_method, name = "victims") %>%
    mutate(percent = round(victims / sum(victims) * 100, 1)) %>%
    arrange(desc(victims)),
  row.names = FALSE
)
