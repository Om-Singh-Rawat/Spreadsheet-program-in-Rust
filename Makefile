# ===== File Paths =====
EXECUTABLE = target/release/sheet
REPORT_SRC = report/report.tex
REPORT_PDF = report/report.pdf

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
	cargo test --test integration

# Run unit tests with tarpaulin for coverage (only for main.rs, excluding GUI)
coverage:
	cargo tarpaulin --out Html --exclude-files src/spreadsheet_ui/*

coverage-open:
	xdg-open tarpaulin-report.html


# ===== Report Generation =====
report: $(REPORT_PDF)

$(REPORT_PDF): $(REPORT_SRC)
	pdflatex -output-directory=report $(REPORT_SRC)

# ===== Clean-Up =====
clean:
	cargo clean
	rm -f report/*.aux report/*.log report/*.pdf
	rm -f tarpaulin-report.html