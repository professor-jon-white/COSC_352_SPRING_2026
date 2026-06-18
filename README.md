# COSC 352 Spring 2026

Coursework repository for COSC 352, collecting programming assignments, Docker-based exercises, data parsing tools, grading scripts, and language exploration projects from the Spring 2026 semester.

## Overview

This repository contains class projects organized by student folders, plus shared grading utilities and fixtures. The work covers command-line programming, containerized execution, HTML table parsing, automated grading, data analysis, and multi-language implementation practice.

Sandeep Shah's work is organized under:

```text
sandeep_shah/
```

## Highlighted Work

| Project | Description | Main Tools |
| --- | --- | --- |
| `sandeep_shah/project02` | HTML table parsing and CSV export | Python |
| `sandeep_shah/project04` | Prime counting implementations across languages | Java, Kotlin, Go |
| `sandeep_shah/project05` | Histogram generation and containerized R workflow | R, Docker |
| `sandeep_shah/project06` | Data scraping and dashboard workflow | R |
| `sandeep_shah/project07/csvprof` | CSV profiling CLI with statistics and reports | Rust |
| `sandeep_shah/project08` | Extended Rust CSV profiling and data analysis tools | Rust |

## Repository Structure

```text
COSC_352_SPRING_2026/
├── grading/                 # Shared grading scripts and fixtures
├── project03/               # Autograder tests and scripts
├── sandeep_shah/            # Sandeep Shah's coursework
│   ├── project02/
│   ├── project04/
│   ├── project05/
│   ├── project06/
│   ├── project07/
│   └── project08/
└── <student_name>/          # Other student submission folders
```

## Skills Demonstrated

- Python scripting and CSV generation
- HTML table extraction
- Docker-based project execution
- Shell scripting for repeatable workflows
- Rust CLI development
- Data profiling and summary statistics
- R visualization and dashboard work
- Automated grading workflows

## Running Projects

Each project folder may have its own instructions. Start with the project-level `README.md` when available.

Examples:

```bash
cd sandeep_shah/project07/csvprof
cargo run -- sample.csv
```

```bash
cd sandeep_shah/project05
./run.sh
```

## Notes

- Some folders contain class-wide grading assets or other student submissions.
- Generated files, build outputs, and local environment files should stay out of version control.
- Project-specific setup steps vary by language and assignment.

## Author

Sandeep Shah
