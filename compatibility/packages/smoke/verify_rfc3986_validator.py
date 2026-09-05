"""Accept HTTPS/mailto URIs and relative references; reject a relative URI under the absolute rule and an unescaped space."""
from rfc3986_validator import validate_rfc3986
assert validate_rfc3986("https://example.com/path?q=hello%20world#part")
assert validate_rfc3986("mailto:hello@example.com")
assert validate_rfc3986("../images/logo.png", rule="URI_reference")
assert not validate_rfc3986("../images/logo.png")
assert not validate_rfc3986("https://example.com/has a space")
print("verified rfc3986-validator")
