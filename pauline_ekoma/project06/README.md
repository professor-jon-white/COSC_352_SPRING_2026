Selected Statistics
For the 2025 Baltimore City homicide list, I examined the age distribution of the victims to identify demographic trends.

Reasoning Behind My Pick
The distribution of ages is a critical metric for determining if certain demographic groups are disproportionately affected by violence. By utilizing 5-year bins, the dashboard's histogram illustrates clustering patterns that indicate whether homicides are heavily concentrated among younger adults or distributed more broadly.

Data Cleaning Notes
Extraction: Employed Regex to accurately parse ages from victim entry strings.
Validation: Eliminated rows with missing or invalid age data to ensure a clean dataset.
Aggregation: Grouped ages into 5-year increments; the tabular display is configured to show only non-empty bins for concise reporting.

Dashboard Features
Geospatial Mapping: A leaflet map using markerClusterOptions() to show incident density across Baltimore neighborhoods.
Reactive Filtering: A sidebar allowing users to filter the entire dashboard by Date Range and Method (e.g., weapon type).
Trend Visualization: An interactive plotly line chart showing incident frequency over time.
Key Performance Indicators: valueBox displays providing immediate totals for incidents based on user-selected filters.