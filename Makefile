# Makefile for the Rust Spreadsheet Project

# Executable name
EXEC = sheet

# Report file
REPORT_SRC = report/report.tex
REPORT_PDF = report/report.pdf

# Default target: build the release binary and create ./sheet
all: build

build:
	cargo build --release
	cp target/release/spreadsheet $(EXEC)

# Run tests (currently works even if tests are empty)
test:
	cargo test

# Compile the LaTeX report to PDF
report: $(REPORT_PDF)

$(REPORT_PDF): $(REPORT_SRC)
	pdflatex -output-directory=report $(REPORT_SRC)

# Clean build artifacts and generated files
clean:
	cargo clean
	rm -f $(EXEC)
	rm -f report/*.aux report/*.log report/*.pdf
