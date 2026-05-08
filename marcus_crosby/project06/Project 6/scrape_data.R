#!/usr/bin/env Rscript

suppressPackageStartupMessages({
  library(rvest)
  library(xml2)
  library(dplyr)
  library(stringr)
  library(lubridate)
  library(jsonlite)
})

source_url <- "https://chamspage.blogspot.com/2025/01/2025-baltimore-city-homicide-list.html"
output_path <- "homicide_data.csv"
geocode_addresses <- TRUE

args <- commandArgs(trailingOnly = TRUE)
if (length(args) >= 1 && nzchar(args[[1]])) {
  source_url <- args[[1]]
}
if (length(args) >= 2 && nzchar(args[[2]])) {
  output_path <- args[[2]]
}
if (length(args) >= 3 && nzchar(args[[3]])) {
  geocode_addresses <- tolower(args[[3]]) %in% c("true", "t", "yes", "y", "1", "geocode")
}

clean_text <- function(x) {
  x |>
    str_replace_all("[\r\n\t]+", " ") |>
    str_replace_all("\\s+", " ") |>
    str_trim()
}

empty_to_na <- function(x) {
  x <- clean_text(x)
  ifelse(x == "" | x %in% c("NA", "N/A"), NA_character_, x)
}

parse_death_date <- function(x) {
  x <- str_remove(clean_text(x), "\\s*\\([^)]*\\)\\s*$")

  parsed <- suppressWarnings(case_when(
    str_detect(x, "^\\d{1,2}/\\d{1,2}/\\d{2,4}$") ~ as.Date(mdy(x)),
    str_detect(x, "^\\d{1,2}/\\d{4}$") ~ as.Date(my(x)),
    TRUE ~ as.Date(NA)
  ))

  parsed
}

extract_time <- function(x) {
  time <- str_match(x, "\\(([^)]*\\d{1,2}:?\\d{0,2}\\s*(?:am|pm|AM|PM)?[^)]*)\\)")[, 2]
  empty_to_na(time)
}

infer_method <- function(notes) {
  notes_lower <- str_to_lower(coalesce(notes, ""))

  case_when(
    str_detect(notes_lower, "shoot|shot|gunshot") ~ "Shooting",
    str_detect(notes_lower, "stab") ~ "Stabbing",
    str_detect(notes_lower, "strang") ~ "Strangulation",
    str_detect(notes_lower, "blunt") ~ "Blunt force",
    str_detect(notes_lower, "fire|arson|burn") ~ "Fire",
    str_detect(notes_lower, "vehicle|crash|collision") ~ "Vehicle",
    str_detect(notes_lower, "trauma") ~ "Trauma",
    TRUE ~ "Unknown"
  )
}

safe_first_href <- function(cell) {
  href <- html_element(cell, "a") |> html_attr("href")
  if (length(href) == 0 || is.na(href) || href == "") NA_character_ else href
}

safe_cell_text <- function(cell) {
  html_text2(cell) |> clean_text()
}

extract_homicide_table <- function(url) {
  page <- read_html(url)
  table_node <- html_element(page, "#homicidelist")

  if (length(table_node) == 0 || inherits(table_node, "xml_missing")) {
    stop("Could not find the table with id='homicidelist' at the source URL.")
  }

  rows <- xml_find_all(table_node, "./tbody/tr")
  if (length(rows) < 2) {
    stop("The homicide table did not contain the expected body rows.")
  }

  col_names <- c(
    "case_number_raw",
    "date_died_raw",
    "victim_name",
    "age_raw",
    "address",
    "notes",
    "violent_history_raw",
    "cctv_raw",
    "case_closed_raw"
  )

  records <- lapply(rows[-1], function(row) {
    cells <- xml_find_all(row, "./td")
    if (length(cells) != length(col_names)) {
      return(NULL)
    }

    values <- vapply(cells, safe_cell_text, character(1))
    names(values) <- col_names

    c(values, source_link = safe_first_href(cells[[3]]))
  })

  records <- Filter(Negate(is.null), records)
  raw_df <- as_tibble(as.data.frame(do.call(rbind, records), stringsAsFactors = FALSE))

  raw_df |>
    mutate(across(everything(), empty_to_na)) |>
    filter(if_any(everything(), ~ !is.na(.x))) |>
    mutate(
      case_number = suppressWarnings(as.integer(str_extract(case_number_raw, "\\d+"))),
      is_counted_case = str_detect(coalesce(case_number_raw, ""), "^\\s*\\d+\\s*$"),
      date_died = parse_death_date(date_died_raw),
      date_is_approximate = str_detect(coalesce(date_died_raw, ""), "^\\d{1,2}/\\d{4}$"),
      time_of_death = extract_time(coalesce(date_died_raw, "")),
      year = year(date_died),
      month = month(date_died),
      month_name = as.character(month(date_died, label = TRUE, abbr = TRUE)),
      victim_age = suppressWarnings(as.integer(str_extract(age_raw, "\\d+"))),
      method = infer_method(notes),
      camera_count = suppressWarnings(as.integer(str_extract(coalesce(cctv_raw, ""), "\\d+"))),
      camera_count = coalesce(camera_count, 0L),
      has_cctv = camera_count > 0,
      case_closed = str_detect(str_to_lower(coalesce(case_closed_raw, "")), "closed"),
      no_violent_history = case_when(
        str_detect(str_to_lower(coalesce(violent_history_raw, "")), "none|no") ~ TRUE,
        is.na(violent_history_raw) ~ NA,
        TRUE ~ FALSE
      ),
      scrape_source_url = source_url,
      scraped_at = format(Sys.time(), "%Y-%m-%d %H:%M:%S %Z")
    ) |>
    select(
      case_number,
      case_number_raw,
      is_counted_case,
      date_died,
      date_died_raw,
      date_is_approximate,
      time_of_death,
      year,
      month,
      month_name,
      victim_name,
      victim_age,
      address,
      method,
      notes,
      camera_count,
      has_cctv,
      case_closed,
      no_violent_history,
      violent_history_raw,
      cctv_raw,
      case_closed_raw,
      source_link,
      scrape_source_url,
      scraped_at
    )
}

geocode_one <- function(address) {
  if (is.na(address) || !nzchar(address)) {
    return(tibble(latitude = NA_real_, longitude = NA_real_, matched_address = NA_character_))
  }

  query <- paste(address, "Baltimore", "MD")
  endpoint <- sprintf(
    "https://geocoding.geo.census.gov/geocoder/locations/onelineaddress?address=%s&benchmark=Public_AR_Current&format=json",
    URLencode(query, reserved = TRUE)
  )

  result <- tryCatch(
    fromJSON(endpoint),
    error = function(e) NULL
  )

  matches <- tryCatch(result$result$addressMatches, error = function(e) NULL)

  if (is.null(result) || is.null(matches) || length(matches) == 0 || is.null(nrow(matches)) || nrow(matches) == 0) {
    return(tibble(latitude = NA_real_, longitude = NA_real_, matched_address = NA_character_))
  }

  first_match <- matches[1, ]
  tibble(
    latitude = as.numeric(first_match$coordinates$y),
    longitude = as.numeric(first_match$coordinates$x),
    matched_address = first_match$matchedAddress
  )
}

add_coordinates <- function(df) {
  addresses <- df |>
    distinct(address) |>
    filter(!is.na(address))

  if (nrow(addresses) == 0) {
    return(mutate(df, latitude = NA_real_, longitude = NA_real_, matched_address = NA_character_))
  }

  message(sprintf("Geocoding %d unique Baltimore addresses with the Census geocoder...", nrow(addresses)))

  coords <- lapply(addresses$address, function(addr) {
    Sys.sleep(0.1)
    bind_cols(tibble(address = addr), geocode_one(addr))
  }) |>
    bind_rows()

  df |>
    left_join(coords, by = "address")
}

message(sprintf("Scraping %s", source_url))
homicides <- extract_homicide_table(source_url)

if (geocode_addresses) {
  homicides <- add_coordinates(homicides)
} else {
  homicides <- homicides |>
    mutate(latitude = NA_real_, longitude = NA_real_, matched_address = NA_character_)
}

homicides <- homicides |>
  relocate(latitude, longitude, matched_address, .after = address)

write.csv(homicides, output_path, row.names = FALSE, na = "")

message(sprintf(
  "Wrote %d rows (%d counted 2025 cases) to %s",
  nrow(homicides),
  sum(homicides$is_counted_case, na.rm = TRUE),
  normalizePath(output_path, mustWork = FALSE)
))
