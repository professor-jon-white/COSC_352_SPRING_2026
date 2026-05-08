FROM r-base:latest

ENV R_LIBS_USER=/usr/local/lib/R/site-library

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    cmake \
    gdal-bin \
    libcurl4-openssl-dev \
    libfontconfig1-dev \
    libfreetype6-dev \
    libfribidi-dev \
    libgdal-dev \
    libgeos-dev \
    libharfbuzz-dev \
    libicu-dev \
    libjpeg-dev \
    libpng-dev \
    libproj-dev \
    libssl-dev \
    libtiff-dev \
    libudunits2-dev \
    libxml2-dev \
    pandoc \
    zlib1g-dev \
  && rm -rf /var/lib/apt/lists/*

RUN R -e "install.packages(c('shiny', 'rvest', 'xml2', 'dplyr', 'stringr', 'lubridate', 'jsonlite', 'ggplot2', 'plotly', 'leaflet', 'DT', 'bslib', 'scales', 'htmltools'), repos = 'https://cloud.r-project.org')"

WORKDIR /srv/shiny-server

COPY scrape_data.R app.R homicide_data.csv ./

EXPOSE 3838

CMD ["R", "-e", "shiny::runApp('/srv/shiny-server', host = '0.0.0.0', port = 3838)"]
