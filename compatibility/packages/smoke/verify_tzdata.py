"""Verify release/IANA constants and TZif headers for four named timezone files."""
import os
import tzdata
assert tzdata.__version__ == "2026.3"
assert tzdata.IANA_VERSION == "2026c"
for name in ["UTC", "Europe/Berlin", "America/New_York", "Asia/Tokyo"]:
    with open(os.path.join(os.path.dirname(tzdata.__file__), "zoneinfo", name), "rb") as zone:
        assert zone.read(4) == b"TZif"
print("verified tzdata")
