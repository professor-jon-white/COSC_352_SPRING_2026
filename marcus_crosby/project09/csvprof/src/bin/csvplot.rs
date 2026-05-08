use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context};
use clap::Parser;

use csvprof::plot::{
    load_csv_plot_data, load_csv_plot_file, render_csv_plot, CsvPlotConfig, PlotKind,
    PlotRenderConfig,
};

#[derive(Debug, Parser)]
#[command(
    name = "csvplot",
    version,
    about = "Render CSV columns as PNG or SVG charts using plotters"
)]
struct Cli {
    /// Input CSV path. Use `-` to read from stdin.
    file: PathBuf,
    /// Output image path. Use `.svg` for SVG; any other extension renders PNG.
    #[arg(short, long)]
    output: PathBuf,
    /// X column name, `column_N`, or 1-based index.
    #[arg(long)]
    x: String,
    /// Y column name, `column_N`, or 1-based index.
    #[arg(long)]
    y: String,
    /// Chart type to render.
    #[arg(long, value_enum, default_value_t = PlotKind::Scatter)]
    kind: PlotKind,
    /// Optional chart title.
    #[arg(long)]
    title: Option<String>,
    /// CSV delimiter as a single-byte character.
    #[arg(long, default_value = ",")]
    delimiter: char,
    /// Treat the first row as data instead of headers.
    #[arg(long)]
    no_headers: bool,
    /// Maximum rows to read for plotting.
    #[arg(long, default_value_t = 5000)]
    limit: usize,
    /// Number of largest values to keep for bar charts.
    #[arg(long, default_value_t = 25)]
    top: usize,
    /// Output image width in pixels.
    #[arg(long, default_value_t = 1200)]
    width: u32,
    /// Output image height in pixels.
    #[arg(long, default_value_t = 800)]
    height: u32,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = cli.plot_config()?;

    let dataset = if cli.file == Path::new("-") {
        let reader = open_stdin();
        load_csv_plot_data(reader, &config).context("failed to load plot data from stdin")?
    } else {
        load_csv_plot_file(&cli.file, &config).with_context(|| {
            format!(
                "failed to load plot data from `{}`",
                cli.file.as_os_str().to_string_lossy()
            )
        })?
    };

    render_csv_plot(
        &dataset,
        &cli.output,
        &PlotRenderConfig {
            width: cli.width,
            height: cli.height,
            title: cli.title.clone(),
        },
    )
    .with_context(|| format!("failed to render `{}`", cli.output.display()))?;

    eprintln!(
        "csvplot summary: plotted {} rows, skipped {} rows, wrote {}",
        dataset.rows.len(),
        dataset.skipped_rows,
        cli.output.display()
    );
    Ok(())
}

impl Cli {
    fn plot_config(&self) -> anyhow::Result<CsvPlotConfig> {
        let delimiter = u8::try_from(self.delimiter as u32)
            .with_context(|| format!("invalid single-byte delimiter `{}`", self.delimiter))?;
        ensure!(self.width >= 320, "plot width must be at least 320 pixels");
        ensure!(
            self.height >= 240,
            "plot height must be at least 240 pixels"
        );

        Ok(CsvPlotConfig {
            delimiter,
            has_headers: !self.no_headers,
            x_column: self.x.clone(),
            y_column: self.y.clone(),
            kind: self.kind,
            limit: self.limit,
            top: self.top,
        })
    }
}

fn open_stdin() -> Box<dyn Read> {
    Box::new(BufReader::new(io::stdin()))
}
