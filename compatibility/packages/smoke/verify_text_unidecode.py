"""Check accented Latin, Cyrillic, Chinese, ASCII, and empty-input transliteration using the bundled data table."""
from text_unidecode import unidecode
assert unidecode("café déjà vu") == "cafe deja vu"
assert unidecode("Привет") == "Privet"
assert unidecode("北京") == "Bei Jing "
assert unidecode("plain ASCII 123") == "plain ASCII 123"
assert unidecode("") == ""
print("verified text-unidecode")
