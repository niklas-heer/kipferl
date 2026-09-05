"""Test tzdata's published version constants and representative TZif assets.

This is resource compatibility, not a claim that Kipferl implements zoneinfo.
The source code imported from this wheel consists only of version constants
and empty package initializers. No timezone parsing is performed here.
"""
import tzdata

assert tzdata.__version__ == "2025.2"
assert tzdata.IANA_VERSION == "2025b"
for name in ["UTC", "Europe/Berlin", "America/New_York", "Asia/Tokyo"]:
    with open("tzdata/zoneinfo/" + name, "rb") as resource:
        assert resource.read(4) == b"TZif"
print("tzdata 2025.2: version constants and four TZif resources passed")
