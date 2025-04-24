# ===== File Paths =====
EXECUTABLE = target/release/spreadsheet
REPORT_SRC = report/report.tex
REPORT_PDF = report.pdf
REPORT_DIR = report
RUSTDOC_OUT = target/doc

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
test:
	cargo test

coverage:
	cargo tarpaulin --out Html --exclude-files spreadsheet_ui/
	xdg-open tarpaulin-report.html

# ===== Documentation Generation =====
docs: $(REPORT_PDF)
	cargo doc --document-private-items
	cargo doc --open
	
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
