"""Turn a JSON object of counts into a Markdown report."""
import argparse
import json

parser = argparse.ArgumentParser(description='Turn a JSON object of counts into a Markdown report.')
parser.add_argument("input", help="JSON object mapping labels to values")
parser.add_argument("output", help="Markdown file to create")
args = parser.parse_args()

with open(args.input, "r") as source:
    values = json.load(source)
lines = ["# Summary", "", "| Item | Value |", "| --- | ---: |"]
for label in sorted(values):
    safe_label = str(label).replace("|", "\\|").replace("\n", " ")
    safe_value = str(values[label]).replace("|", "\\|").replace("\n", " ")
    lines.append("| " + safe_label + " | " + safe_value + " |")
with open(args.output, "w") as destination:
    destination.write("\n".join(lines) + "\n")
print("Wrote " + args.output)
