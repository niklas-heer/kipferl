"""Accept leap-day and fractional-offset timestamps; reject an invalid leap-day, invalid hour, and date-only input."""
from rfc3339_validator import validate_rfc3339
assert validate_rfc3339("2024-02-29T12:30:45Z")
assert validate_rfc3339("2026-09-05T12:30:45.123+02:00")
assert not validate_rfc3339("2023-02-29T12:30:45Z")
assert not validate_rfc3339("2026-09-05T25:30:45Z")
assert not validate_rfc3339("2026-09-05")
print("verified rfc3339-validator")
