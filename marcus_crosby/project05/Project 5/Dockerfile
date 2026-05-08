FROM r-base:latest

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libcurl4-openssl-dev \
    libssl-dev \
    libxml2-dev \
  && rm -rf /var/lib/apt/lists/*

RUN Rscript -e "n <- parallel::detectCores(); if (is.na(n)) n <- 2; install.packages(c('rvest', 'dplyr', 'stringr', 'ggplot2', 'lubridate'), repos = 'https://cloud.r-project.org', Ncpus = max(1, n - 1))"

WORKDIR /app
COPY histogram.R /app/histogram.R
RUN mkdir -p /app/output

CMD ["Rscript", "/app/histogram.R"]
