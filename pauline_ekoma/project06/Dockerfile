#use official R Shiny image
FROM rocker/shiny:latest

#install system dependencies for R packages
RUN apt-get update && apt-get install -y \
    libssl-dev \
    libcurl4-openssl-dev \
    libxml2-dev \
    && rm -rf /var/lib/apt/lists/*

#install R packages
RUN R -e "install.packages(c('shinydashboard', 'leaflet', 'plotly', 'dplyr', 'readr', 'DT'), repos='https://cran.rstudio.com/')" \
    && rm -rf /tmp/downloaded_packages

#copy app
COPY app.R /srv/shiny-server/dashboard/

#expose port 3838
EXPOSE 3838

#run shiny server
CMD ["/usr/bin/shiny-server"]