FROM --platform=linux/amd64 rocker/shiny:latest

RUN R -e "install.packages(c('shiny','dplyr','ggplot2'), repos='https://cloud.r-project.org/')"

COPY . /srv/shiny-server/

EXPOSE 3838

CMD ["/init"]
