"""Parse a Unicode IRI and a relative reference; reject whitespace in a malformed IRI."""
from rfc3987_syntax import is_valid_syntax_iri, is_valid_syntax_iri_reference
assert is_valid_syntax_iri("https://例え.テスト/資料")
assert is_valid_syntax_iri_reference("../資料")
assert not is_valid_syntax_iri("https://example.com/has a space")
print("verified rfc3987-syntax")
