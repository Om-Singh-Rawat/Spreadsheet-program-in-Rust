# ===== File Paths =====
EXECUTABLE = target/release/spreadsheet
REPORT_SRC = report/report.tex
REPORT_PDF = report.pdf
REPORT_DIR = report

# ===== Default Build Target =====
all: build

build:
	cargo build --release

run: build
	$(EXECUTABLE) 999 18278

# ===== Linting =====
lint:
	cargo fmt --check && cargo clippy -- -D warnings

# ===== Testing =====
# Run integration tests in the `tests/` directory (ignores main.rs unit tests)
test:
	cargo test

# Run unit tests with tarpaulin for coverage (only for main.rs, excluding GUI)
coverage:
	cargo tarpaulin --out Html --exclude-files spreadsheet_ui/
	xdg-open tarpaulin-report.html

# ===== Report Generation =====
docs: $(REPORT_PDF)

$(REPORT_PDF): $(REPORT_SRC)
	pdflatex -output-directory=$(REPORT_DIR) $(REPORT_SRC)
	cp $(REPORT_DIR)/report.pdf $(REPORT_PDF)

# ===== Clean-Up =====
clean:
	cargo clean
	rm -f report/*.aux report/*.log report/*.out report/*.toc
	rm -f report/report.pdf
	rm -f report.pdf
	rm -f tarpaulin-report.html
