"""Construct an offline parser configuration with explicit dummy credentials and reject zero workers."""
from llama_parse import LlamaParse
from pydantic import ValidationError
parser = LlamaParse(api_key="verification-not-a-real-key", result_type="markdown", num_workers=1, verbose=False)
assert parser.num_workers == 1
assert parser.result_type.value == "markdown"
failed = False
try:
    LlamaParse(api_key="verification-not-a-real-key", num_workers=0, verbose=False)
except ValidationError:
    failed = True
assert failed
print("verified llama-parse")
