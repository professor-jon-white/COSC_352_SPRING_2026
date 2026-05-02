# Project 8: Baltimore City Open Data Analysis

## Dataset 1
**Name:** Baltimore Crime Data  
**SourceURL** https://data.baltimorecity.gov/datasets/baltimore::part-1-crime-data-legacy-srs/about 
**Description:** This dataset contains reported crime incidents across Baltimore neighborhoods, including location and offense information.  
**Key Columns Used:** Neighborhood

---

## Dataset 2
**Name:** Baltimore 911 Calls for Service    
**Description:** This dataset contains 911 service call records by neighborhood.  
**Key Columns Used:** Neighborhood

---

## Research Question
Which Baltimore neighborhoods with the highest reported crime counts also have the highest number of 911 service calls?

---

## Answer
The analysis found that several neighborhoods with high crime counts had high 911 call volumes, but not always proportionally. For example:

- DOWNTOWN had 22,846 crimes and 52 911 calls  
- SANDTOWN-WINCHESTER had 9,015 crimes and 609 911 calls  
- UPTON had 8,118 crimes and 263 911 calls  
- BROOKLYN had 11,984 crimes and 159 911 calls  

This suggests that while some high-crime neighborhoods also generate high emergency service demand, others may differ due to population density, commercial activity, reporting patterns, or policing practices.

---

## Limitations
This analysis only correlates neighborhood-level counts and does not control for:
- population size
- tourism/commercial traffic
- underreporting
- differences in crime type
- time period differences between datasets

Therefore, correlation does not prove causation.

---

Author: Rochak Ghimire