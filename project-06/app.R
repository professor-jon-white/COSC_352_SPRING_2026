library(shiny)
library(dplyr)
library(ggplot2)

# ---------------------------
# Fake/placeholder dataset loader (replace with your scraper if needed)
# ---------------------------
set.seed(123)
data <- data.frame(
  year = sample(2015:2025, 500, replace = TRUE),
  age = sample(15:80, 500, replace = TRUE),
  method = sample(c("Shooting", "Stabbing", "Other"), 500, replace = TRUE),
  cleared = sample(c(0,1), 500, replace = TRUE)
)

ui <- fluidPage(
  titlePanel("Baltimore Homicide Analysis Dashboard"),

  sidebarLayout(
    sidebarPanel(
      sliderInput("year", "Year Range:",
                  min = 2015, max = 2025,
                  value = c(2018, 2025)),

      sliderInput("age", "Victim Age:",
                  min = 10, max = 90,
                  value = c(15, 80)),

      checkboxGroupInput("method", "Method:",
                         choices = c("Shooting", "Stabbing", "Other"),
                         selected = c("Shooting", "Stabbing", "Other"))
    ),

    mainPanel(
      tabsetPanel(
        tabPanel("Overview", plotOutput("trendPlot")),
        tabPanel("Distribution", plotOutput("agePlot")),
        tabPanel("Stats", verbatimTextOutput("summaryStats"))
      )
    )
  )
)

server <- function(input, output) {

  filtered <- reactive({
    data %>%
      filter(year >= input$year[1],
             year <= input$year[2],
             age >= input$age[1],
             age <= input$age[2],
             method %in% input$method)
  })

  output$trendPlot <- renderPlot({
    ggplot(filtered(), aes(year)) +
      geom_histogram(binwidth = 1, fill = "steelblue") +
      theme_minimal() +
      labs(title = "Homicides Over Time")
  })

  output$agePlot <- renderPlot({
    ggplot(filtered(), aes(age)) +
      geom_histogram(binwidth = 5, fill = "darkred") +
      theme_minimal() +
      labs(title = "Victim Age Distribution")
  })

  output$summaryStats <- renderPrint({
    d <- filtered()
    cat("Total Cases:", nrow(d), "\n")
    cat("Clearance Rate:", mean(d$cleared) * 100, "%\n")
    cat("Average Age:", mean(d$age), "\n")
  })
}

shinyApp(ui, server)
