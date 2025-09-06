# /// script
# requires-python = ">=3.13"
# dependencies = [
# ]
# ///

from typing import Optional
import xml.etree.ElementTree as ET
import re
import os
import sys
import subprocess
from pathlib import Path
from dataclasses import dataclass
from datetime import datetime, timezone

import urllib.request

if len(sys.argv) < 2:
    print("Usage: read-libdwd-tables.py <path-to-DWD-bufrtables>")
    exit(1)

existing_table_b = urllib.request.urlopen(
    "https://raw.githubusercontent.com/MIERUNE/tinybufr/master/src/tables/table_b.rs"
).read()
existing_table_d = urllib.request.urlopen(
    "https://raw.githubusercontent.com/MIERUNE/tinybufr/master/src/tables/table_d.rs"
).read()


XY_RE = re.compile(r"XY \{ x: (\d+), y: (\d+) \},")

xy_b = set((0, int(x), int(y)) for x, y in XY_RE.findall(str(existing_table_b)))
xy_d = set((3, int(x), int(y)) for x, y in XY_RE.findall(str(existing_table_d)))

FXY = tuple[int, int, int]

new_xy_b: dict[FXY, "BDescr"] = dict()
new_xy_d: dict[FXY, tuple[str, list[FXY]]] = dict()


@dataclass
class BDescr:
    name: str
    unit: str
    scale: int
    reference: int
    bits: int


def parse_id(id: str) -> FXY:
    return int(id[0]), int(id[1:3]), int(id[3:])


def discover_files(dir: str) -> tuple[list[str], list[str]]:
    if not dir.endswith("tabellen_v3"):
        dir += "/tabellen_v3"
    b_files = []
    d_files = []
    for root, _dirs, files in os.walk(dir):
        for file in files:
            if not file.startswith("table_"):
                continue
            path = os.path.join(root, file)
            with open(path) as f:
                # [...some text...] created: 2021-05-28 11:32:25
                line = f.readline()
            m = re.search(r"created:\s*(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})", line)
            if m:
                dt = datetime.strptime(m.group(1), "%Y-%m-%d %H:%M:%S").replace(
                    tzinfo=timezone.utc
                )
                created_seconds = int(dt.timestamp())
                if file.startswith("table_b"):
                    b_files.append((path, created_seconds))
                elif file.startswith("table_d"):
                    d_files.append((path, created_seconds))

    b_files.sort(key=lambda it: it[1])
    d_files.sort(key=lambda it: it[1])
    return [f for f, _t in b_files], [f for f, _t in d_files]


# first line per sequence: "FXY1<tab>FXY2<lf>"
# following lines per seq: "<tab>FXY2<lf>"
def process_d_file(path: str):
    with open(path) as f:
        lines = f.readlines()
    last_comment = ""
    cur_id: Optional[FXY] = None
    cur_seq: list[FXY] = []

    def flush():
        nonlocal cur_id, cur_seq, last_comment
        if cur_id is not None:
            if cur_id not in xy_d:
                new_xy_d[cur_id] = (last_comment, cur_seq)
            cur_id = None
            cur_seq = []

    for line in lines:
        if len(line) < 3:
            continue
        if line[0] == "#":
            flush()
            last_comment = line[1:].strip()
            continue
        fxy1, fxy2 = line.split("\t", 1)
        if len(fxy1) > 0:
            cur_id = parse_id(fxy1)
        cur_seq.append(parse_id(fxy2))

    flush()


# line format: "FXY<tab>libDWDType<tab>unit<tab>scale<tab>referenceValue<tab>dataWidth_Bits<tab>descriptor_name<lf>"
def process_b_file(path: str):
    with open(path) as f:
        lines = f.readlines()
    for line in lines:
        if line[0] == "#" or len(line) < 3:
            continue
        fxy, _libdwdtype, unit, scale, reference, bits, name = line.split("\t", 6)
        id = parse_id(fxy)
        if id in xy_b:
            continue
        new_xy_b[id] = BDescr(
            name=name.strip(),
            unit=unit,
            scale=int(scale),
            reference=int(reference),
            bits=int(bits),
        )


b_files, d_files = discover_files(sys.argv[1])
for f in b_files:
    process_b_file(f)
for f in d_files:
    process_d_file(f)

print(len(new_xy_b), len(new_xy_d))

OUT_FILE = Path(__file__).parent.parent / "src" / "dwd" / "bufr_tables.rs"
with open(OUT_FILE, "w") as f:
    f.write(
        r"""// This file was automatically generated

use tinybufr::{TableBEntry, TableDEntry, XY, Descriptor};
"""
    )
    f.write(f"pub static DWD_BUFR_TABLE_B: [TableBEntry; {len(new_xy_b)}] = [\n")
    for k, v in new_xy_b.items():
        f.write(
            f"""TableBEntry{{  xy: XY {{ x: {k[1]}, y: {k[2]} }},
            class_name: "dwd",
            element_name: "{v.name.replace('"', '\\"')}",
            unit: "{v.unit}",
            scale: {v.scale},
            reference_value: {v.reference},
            bits: {v.bits},
        }},"""
        )
    f.write("];\n\n")

    f.write(f"pub static DWD_BUFR_TABLE_D: [TableDEntry; {len(new_xy_d)}] = [\n")
    for k, v in new_xy_d.items():
        f.write(
            f"""TableDEntry {{
        xy: XY {{ x: {k[1]}, y: {k[2]} }},
        category: "DWD sequences",
        title: "{v[0].replace('"', '\\"')}",
        sub_title: "",
        elements: &[
            {"\n".join(f"Descriptor {{ f: {f}, x: {x}, y: {y} }}," for f, x, y in v[1])}
        ],
    }},"""
        )
    f.write("];\n")
