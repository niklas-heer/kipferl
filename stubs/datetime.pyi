"""PocketPy date, datetime and timedelta classes.

Only days and seconds are supported by timedelta. datetime requires all six
constructor fields; now uses local time. No timezone, timestamp conversion,
or date arithmetic API is provided by this compatibility module.
"""

class timedelta:
    days: int
    seconds: int
    def __init__(self, days: int = 0, seconds: int = 0) -> None: ...
    def total_seconds(self) -> float: ...

class date:
    year: int
    month: int
    day: int
    def __init__(self, year: int, month: int, day: int) -> None: ...
    @staticmethod
    def today() -> "date": ...
    def isoformat(self) -> str: ...
    def weekday(self) -> int: ...

class datetime(date):
    hour: int
    minute: int
    second: int
    def __init__(self, year: int, month: int, day: int, hour: int, minute: int, second: int) -> None: ...
    def date(self) -> date: ...
    @staticmethod
    def now() -> "datetime": ...
