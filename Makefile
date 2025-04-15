# Makefile for the Rust Spreadsheet Project

# Report files
REPORT_SRC = report/report.tex
REPORT_PDF = report/report.pdf

# Default target: build the release binary
all: build

# Build the release binary (output stays in target/release/)
build:
	cargo build --release

# Run tests from the tests/ folder
test:
	cargo test

# Generate the PDF report from LaTeX
report: $(REPORT_PDF)

$(REPORT_PDF): $(REPORT_SRC)
	pdflatex -output-directory=report $(REPORT_SRC)

# Clean build artifacts and generated files
clean:
	cargo clean
	rm -f report/*.aux report/*.log report/*.pdf
