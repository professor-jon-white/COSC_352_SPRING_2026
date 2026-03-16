library(shiny)
library(shinydashboard)
library(leaflet)
library(plotly)
library(dplyr)

#--- UI ---
ui <- dashboardPage(
    dashboardHeader(title="BPD Crime Analysis"),
    dashboardSidebar(
        sidebarMenu(
            menuItem("Dashboard", tabName="dashboard", icon=icon("dashboard")),
            dateRangeInput("date_range", "Time Period:",
                start=min(data$incident_date),
                end=max(data$incident_date)),
            checkboxGroupInput("method", "Filter by Method:",
                choices=unique(data$method),
                selected=unique(data$method))
        )
    ),
    dashboardBody(
        fluidRow(
            valueBoxOutput("total_homicides", width=5),
            valueBoxOutput("clearance_rate", width=5),
            valueBoxOutput("camera_prox", width=5)
        ),
        fluidRow(
            box(title="Incident Map", status="primary", solidHeader=TRUE,
                leafletOutput("crime_map", height=400)),
            box(title="Trend Over Time", status="warning", solidHeader=TRUE,
                plotlyOutput("trend_plot", height=400))
            
        )
    )
)
#--- Server ---
server <- function(input, output) {
    filtered_data <- reactive({
        data%>%
            filter(incident_date>=input$date_range[1],
            incident_date<=input$date_range[2],
            method %in% input$method)
    })
    output$total_homicides<-renderValueBox({
        valueBox(nrow(filtered_data()), "Total Incidents", icon=icon("list"), color="red")
    })
    output$crime_map<- renderLeaflet({
        leaflet(filtered_data()) %>%
            addTiles() %>%
            addCircleMarkers(~longitude, ~latitude, popup=~paste("Date:", incident_date),
                clusterOptions=markerClusterOptions())
    })
    output$trend_plot<-renderPlotly({
        p<-filtered_data() %>%
            count(incident_date) %>%
            ggplot(aes(x=incident_date, y=n)) +
            geom_line(color="blue") + 
            labs(x="Date", y="Number of Incidents") +
            theme_minimal()
        ggplotly(p)
    })
}
shinyApp(ui, server)