"""
Simplified datetime module tests for kipferl compatibility testing.
Works on both CPython and PocketPy.

Based on CPython's Lib/test/test_datetime.py
"""

import datetime
import sys

# Test tracking
_passed = 0
_failed = 0
_skipped = 0
_errors = []


def test(name, condition):
    global _passed, _failed, _errors
    if condition:
        _passed += 1
        print(f"  PASS: {name}")
    else:
        _failed += 1
        _errors.append(name)
        print(f"  FAIL: {name}")


def skip(name, reason):
    global _skipped
    _skipped += 1
    print(f"  SKIP: {name} ({reason})")


# ============================================================================
# datetime.now() tests - test getting current time
# ============================================================================

print("\n=== datetime.now() tests ===")

now = datetime.datetime.now()
test("now year is reasonable", 2020 <= now.year <= 2100)
test("now month is valid", 1 <= now.month <= 12)
test("now day is valid", 1 <= now.day <= 31)
test("now hour is valid", 0 <= now.hour <= 23)
test("now minute is valid", 0 <= now.minute <= 59)
test("now second is valid", 0 <= now.second <= 59)

# ============================================================================
# date tests - test date creation and isoformat
# ============================================================================

print("\n=== date tests ===")

d1 = datetime.date(2024, 1, 15)
d2 = datetime.date(2000, 12, 31)
test("date_isoformat 2024-01-15", d1.isoformat() == "2024-01-15")
test("date_isoformat 2000-12-31", d2.isoformat() == "2000-12-31")

# ============================================================================
# weekday tests - test weekday calculation
# ============================================================================

print("\n=== weekday tests ===")

# 2024-01-15 is a Monday (weekday 0)
test("weekday monday", datetime.date(2024, 1, 15).weekday() == 0)
# 2024-01-21 is a Sunday (weekday 6)
test("weekday sunday", datetime.date(2024, 1, 21).weekday() == 6)

# ============================================================================
# is_leap_year tests
# ============================================================================

print("\n=== is_leap_year tests ===")


def _is_leap(year):
    return year % 4 == 0 and (year % 100 != 0 or year % 400 == 0)


# Test leap year using local function
test("2000 is leap year", _is_leap(2000) == True)
test("2024 is leap year", _is_leap(2024) == True)
test("2023 is not leap year", _is_leap(2023) == False)
test("1900 is not leap year", _is_leap(1900) == False)

# ============================================================================
# timedelta tests
# ============================================================================

print("\n=== timedelta tests ===")

td = datetime.timedelta(days=1, seconds=3600)
test("timedelta days", td.days == 1)
test("timedelta seconds", td.seconds == 3600)

# ============================================================================
# timedelta total_seconds tests
# ============================================================================

print("\n=== timedelta total_seconds tests ===")

td = datetime.timedelta(days=1, seconds=3600)
# 1 day + 3600 seconds = 86400 + 3600 = 90000 seconds
test("timedelta_total_seconds", td.total_seconds() == 90000.0)

# ============================================================================
# days_in_month tests
# ============================================================================

print("\n=== days_in_month tests ===")


def _days_in_month(year, month):
    days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    if month == 2 and _is_leap(year):
        return 29
    return days[month]


# Test days in month using local function
test("january has 31 days", _days_in_month(2024, 1) == 31)
test("feb leap has 29 days", _days_in_month(2024, 2) == 29)
test("feb non-leap has 28 days", _days_in_month(2023, 2) == 28)
test("april has 30 days", _days_in_month(2024, 4) == 30)

# ============================================================================
# Summary
# ============================================================================

print("\n" + "=" * 50)
print(f"Results: {_passed} passed, {_failed} failed, {_skipped} skipped")
if _errors:
    print("Failed tests:")
    for e in _errors:
        print(f"  - {e}")
    sys.exit(1)
