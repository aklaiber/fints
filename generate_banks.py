#!/usr/bin/env python3
"""
Generate backend/fints/src/banks_generated.rs from the HBCI/FinTS institute CSV.

Usage:
    python3 generate_banks.py <path-to-csv>

The CSV is semicolon-separated and uses Windows-1252 encoding.
Columns (0-indexed):
  0: Nr.
  1: BLZ
  2: BIC
  3: Institut (name)
  4: Ort
  5-23: various HBCI access columns
  24: PIN/TAN-Zugang URL
  25: Version
  26: Datum letzte Änderung
"""

import sys
import csv
import os
from collections import OrderedDict


def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <path-to-csv>", file=sys.stderr)
        sys.exit(1)

    csv_path = sys.argv[1]
    script_dir = os.path.dirname(os.path.abspath(__file__))
    out_path = os.path.join(script_dir, "src", "banks_generated.rs")

    # Parse CSV: semicolon-separated, Windows-1252
    banks = OrderedDict()  # blz -> (blz, bic, name, url)

    with open(csv_path, encoding="windows-1252", errors="replace", newline="") as f:
        reader = csv.reader(f, delimiter=";")
        header = next(reader, None)  # skip header

        for row in reader:
            if len(row) < 25:
                continue

            blz = row[1].strip()
            bic = row[2].strip()
            name = row[3].strip()
            url = row[24].strip()

            # Skip rows without a PIN/TAN URL
            if not url or not url.startswith("http"):
                continue

            # Skip rows without a valid BLZ (8 digits)
            if not blz or len(blz) != 8 or not blz.isdigit():
                continue

            # Deduplicate by BLZ: first occurrence wins for name/BIC
            if blz not in banks:
                banks[blz] = (blz, bic, name, url)

    entries = list(banks.values())
    print(f"Found {len(entries)} unique banks with PIN/TAN URLs", file=sys.stderr)

    # Escape a string for inclusion in a Rust string literal
    def escape(s):
        return s.replace("\\", "\\\\").replace('"', '\\"')

    # Generate Rust source
    lines = []
    lines.append("// AUTO-GENERATED — do not edit by hand.")
    lines.append("// Regenerate with: python3 generate_banks.py <path-to-csv>")
    lines.append("//")
    lines.append(
        f"// Contains {len(entries)} unique German banks with FinTS PIN/TAN access."
    )
    lines.append("")
    lines.append("use crate::banks::BankConfig;")
    lines.append("")
    lines.append(f"/// Total number of banks in the generated registry.")
    lines.append(f"pub const BANK_COUNT: usize = {len(entries)};")
    lines.append("")

    # Generate static array: (blz, bic, name, url)
    lines.append("/// All known German banks with FinTS PIN/TAN access.")
    lines.append("/// Each entry is (blz, bic, name, url).")
    lines.append("pub static GENERATED_BANKS: &[(&str, &str, &str, &str)] = &[")
    for blz, bic, name, url in entries:
        lines.append(
            f'    ("{escape(blz)}", "{escape(bic)}", "{escape(name)}", "{escape(url)}"),'
        )
    lines.append("];")
    lines.append("")

    # Generate match-based lookup by BLZ
    lines.append("/// Look up a bank by BLZ using a compile-time optimized match.")
    lines.append("pub fn generated_bank_by_blz(blz: &str) -> Option<BankConfig> {")
    lines.append("    let (blz, bic, name, url) = match blz {")
    for blz, bic, name, url in entries:
        lines.append(
            f'        "{escape(blz)}" => ("{escape(blz)}", "{escape(bic)}", "{escape(name)}", "{escape(url)}"),'
        )
    lines.append("        _ => return None,")
    lines.append("    };")
    lines.append("    Some(BankConfig::new_raw(blz, bic, name, url))")
    lines.append("}")
    lines.append("")

    # Generate all_banks function
    lines.append("/// Return all generated banks as BankConfig instances.")
    lines.append("pub fn generated_all_banks() -> Vec<BankConfig> {")
    lines.append("    GENERATED_BANKS")
    lines.append("        .iter()")
    lines.append(
        "        .map(|(blz, bic, name, url)| BankConfig::new_raw(*blz, *bic, *name, *url))"
    )
    lines.append("        .collect()")
    lines.append("}")

    with open(out_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")

    print(f"Written: {out_path}", file=sys.stderr)
    print(f"Banks: {len(entries)}", file=sys.stderr)


if __name__ == "__main__":
    main()
