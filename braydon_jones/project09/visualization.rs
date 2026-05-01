let results = analysis::run_profiler(&file1, &file2)?;

let data = results.into_iter().map(|(key, value)| visualize::CorrelatedData {key, value}).collect::<Vec<_>>();
visualize::plot_correlation(&data, "data.png")?;