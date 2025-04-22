# Makefile for the Rust Spreadsheet Project

# ===== File Paths =====
REPORT_SRC = report/report.tex
REPORT_PDF = report/report.pdf

# ===== Build Targets =====
all: build

build:
	cargo build --release

run:
	cargo run --release -- 999 18278

# ===== Code Quality =====
lint:
	cargo fmt --check && cargo clippy -- -D warnings

test:
	cargo test

# ===== Report Generation =====
report: $(REPORT_PDF)

$(REPORT_PDF): $(REPORT_SRC)
	pdflatex -output-directory=report $(REPORT_SRC)

# ===== Clean-Up =====
clean:
	cargo clean
	rm -f report/*.aux report/*.log report/*.pdf
