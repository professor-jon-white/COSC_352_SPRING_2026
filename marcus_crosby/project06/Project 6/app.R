library(shiny)
library(dplyr)
library(stringr)
library(lubridate)
library(ggplot2)
library(plotly)
library(leaflet)
library(DT)
library(bslib)
library(scales)

data_path <- "homicide_data.csv"

if (!file.exists(data_path)) {
  stop("homicide_data.csv was not found. Run Rscript scrape_data.R before starting the app.")
}

raw_homicides <- read.csv(data_path, stringsAsFactors = FALSE, na.strings = c("", "NA")) |>
  mutate(
    date_died = as.Date(date_died),
    month_name = factor(month_name, levels = month.abb, ordered = TRUE),
    method = if_else(is.na(method) | method == "", "Unknown", method),
    case_closed = as.logical(case_closed),
    has_cctv = as.logical(has_cctv),
    is_counted_case = as.logical(is_counted_case),
    victim_age = suppressWarnings(as.integer(victim_age)),
    camera_count = suppressWarnings(as.integer(camera_count)),
    camera_count = coalesce(camera_count, 0L)
  )

available_methods <- sort(unique(raw_homicides$method))
available_years <- sort(unique(raw_homicides$year[!is.na(raw_homicides$year)]))
age_bounds <- range(raw_homicides$victim_age, na.rm = TRUE)
date_bounds <- range(raw_homicides$date_died, na.rm = TRUE)

if (!all(is.finite(age_bounds))) {
  age_bounds <- c(0, 100)
}
if (any(is.na(date_bounds))) {
  date_bounds <- as.Date(c("2025-01-01", "2025-12-31"))
}

theme <- bs_theme(
  version = 5,
  bootswatch = "flatly",
  primary = "#245f73",
  secondary = "#6a6f4c"
)

ui <- page_sidebar(
  title = "Baltimore City Homicide Analysis Dashboard",
  theme = theme,
  fillable = TRUE,
  sidebar = sidebar(
    width = 330,
    dateRangeInput(
      "date_range",
      "Date of death",
      start = date_bounds[[1]],
      end = date_bounds[[2]],
      min = date_bounds[[1]],
      max = date_bounds[[2]]
    ),
    sliderInput(
      "age_range",
      "Victim age",
      min = floor(age_bounds[[1]]),
      max = ceiling(age_bounds[[2]]),
      value = c(floor(age_bounds[[1]]), ceiling(age_bounds[[2]])),
      step = 1
    ),
    checkboxGroupInput(
      "methods",
      "Method",
      choices = available_methods,
      selected = available_methods
    ),
    checkboxInput("counted_only", "Counted 2025 cases only", value = TRUE),
    checkboxInput("closed_only", "Closed cases only", value = FALSE),
    checkboxInput("cctv_only", "CCTV nearby only", value = FALSE)
  ),
  layout_columns(
    value_box(
      title = "Filtered Homicides",
      value = textOutput("total_cases")
    ),
    value_box(
      title = "Clearance Rate",
      value = textOutput("clearance_rate")
    ),
    value_box(
      title = "Average Victim Age",
      value = textOutput("avg_age")
    ),
    value_box(
      title = "CCTV Coverage",
      value = textOutput("cctv_rate")
    ),
    col_widths = c(3, 3, 3, 3)
  ),
  layout_columns(
    card(
      full_screen = TRUE,
      card_header("Victim Age Distribution"),
      plotlyOutput("age_histogram", height = "330px")
    ),
    card(
      full_screen = TRUE,
      card_header("Method by Month"),
      plotlyOutput("method_month_bar", height = "330px")
    ),
    col_widths = c(6, 6)
  ),
  layout_columns(
    card(
      full_screen = TRUE,
      card_header("Incident Locations"),
      leafletOutput("incident_map", height = "430px")
    ),
    card(
      full_screen = TRUE,
      card_header("Filtered Case Records"),
      DTOutput("case_table")
    ),
    col_widths = c(7, 5)
  )
)

server <- function(input, output, session) {
  filtered_data <- reactive({
    req(input$date_range, input$age_range, input$methods)

    raw_homicides |>
      filter(
        is.na(date_died) | (date_died >= input$date_range[[1]] & date_died <= input$date_range[[2]]),
        is.na(victim_age) | (victim_age >= input$age_range[[1]] & victim_age <= input$age_range[[2]]),
        method %in% input$methods
      ) |>
      filter(if (isTRUE(input$counted_only)) is_counted_case else TRUE) |>
      filter(if (isTRUE(input$closed_only)) case_closed else TRUE) |>
      filter(if (isTRUE(input$cctv_only)) has_cctv else TRUE)
  })

  output$total_cases <- renderText({
    comma(nrow(filtered_data()))
  })

  output$clearance_rate <- renderText({
    df <- filtered_data()
    if (nrow(df) == 0) {
      return("0%")
    }
    percent(mean(df$case_closed, na.rm = TRUE), accuracy = 0.1)
  })

  output$avg_age <- renderText({
    avg <- mean(filtered_data()$victim_age, na.rm = TRUE)
    if (is.nan(avg)) "N/A" else number(avg, accuracy = 0.1)
  })

  output$cctv_rate <- renderText({
    df <- filtered_data()
    if (nrow(df) == 0) {
      return("0%")
    }
    percent(mean(df$has_cctv, na.rm = TRUE), accuracy = 0.1)
  })

  output$age_histogram <- renderPlotly({
    df <- filtered_data() |>
      filter(!is.na(victim_age))

    p <- ggplot(df, aes(x = victim_age)) +
      geom_histogram(binwidth = 5, boundary = 0, fill = "#245f73", color = "white") +
      labs(x = "Victim age", y = "Homicides") +
      theme_minimal(base_size = 13)

    ggplotly(p, tooltip = c("x", "y")) |>
      layout(margin = list(l = 55, r = 20, t = 15, b = 45))
  })

  output$method_month_bar <- renderPlotly({
    df <- filtered_data() |>
      filter(!is.na(month_name)) |>
      count(month_name, method, name = "cases")

    p <- ggplot(df, aes(x = month_name, y = cases, fill = method)) +
      geom_col(position = "stack") +
      scale_fill_brewer(palette = "Set2") +
      labs(x = NULL, y = "Homicides", fill = "Method") +
      theme_minimal(base_size = 13) +
      theme(legend.position = "bottom")

    ggplotly(p, tooltip = c("x", "y", "fill")) |>
      layout(margin = list(l = 55, r = 20, t = 15, b = 65))
  })

  output$incident_map <- renderLeaflet({
    df <- filtered_data() |>
      filter(!is.na(latitude), !is.na(longitude))

    map <- leaflet(df) |>
      addProviderTiles(providers$CartoDB.Positron) |>
      setView(lng = -76.6122, lat = 39.2904, zoom = 11)

    if (nrow(df) == 0) {
      return(map)
    }

    popup <- sprintf(
      "<strong>%s</strong><br/>%s<br/>%s<br/>Method: %s<br/>Closed: %s<br/>CCTV cameras: %s",
      htmltools::htmlEscape(df$victim_name),
      htmltools::htmlEscape(df$date_died_raw),
      htmltools::htmlEscape(df$address),
      htmltools::htmlEscape(df$method),
      ifelse(df$case_closed, "Yes", "No"),
      df$camera_count
    )

    map |>
      addCircleMarkers(
        lng = ~longitude,
        lat = ~latitude,
        radius = ~pmax(5, 4 + camera_count),
        stroke = TRUE,
        color = "#1f4e5f",
        weight = 1,
        fillColor = ~ifelse(case_closed, "#2a9d8f", "#d1495b"),
        fillOpacity = 0.78,
        popup = popup,
        clusterOptions = markerClusterOptions()
      ) |>
      addLegend(
        position = "bottomright",
        colors = c("#2a9d8f", "#d1495b"),
        labels = c("Closed", "Open/unknown"),
        title = "Case Status"
      )
  })

  output$case_table <- renderDT({
    filtered_data() |>
      transmute(
        Case = case_number_raw,
        Date = date_died_raw,
        Victim = victim_name,
        Age = victim_age,
        Address = address,
        Method = method,
        CCTV = camera_count,
        Closed = ifelse(case_closed, "Yes", "No"),
        Notes = notes
      ) |>
      datatable(
        rownames = FALSE,
        filter = "top",
        options = list(
          pageLength = 8,
          scrollX = TRUE,
          order = list(list(1, "desc"))
        )
      )
  })
}

shinyApp(ui, server)
