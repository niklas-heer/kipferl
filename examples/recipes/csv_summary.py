"""Summarize a CSV with category and amount columns."""
import argparse
import csv
import json

parser = argparse.ArgumentParser(description='Summarize a CSV with category and amount columns.')
parser.add_argument("file", help="CSV file with category and amount columns")
args = parser.parse_args()

totals = {}
with open(args.file, "r") as source:
    lines = [line.rstrip("\r") for line in source.read().split("\n") if line]
    for row in csv.DictReader(lines):
        category = row["category"]
        totals[category] = totals.get(category, 0.0) + float(row["amount"])
print(json.dumps(totals, sort_keys=True))
