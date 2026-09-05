import os
import tzdata

path = os.path.join(os.path.dirname(tzdata.__file__), "zoneinfo/UTC")
with open(path, "rb") as zone:
    assert zone.read(4) == b"TZif"
print("tzdata " + tzdata.__version__ + ": UTC data bundled")
