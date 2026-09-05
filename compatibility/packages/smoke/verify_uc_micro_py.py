"""Match control, punctuation, ASCII space, and ideographic space categories and reject representative letters."""
import re
from uc_micro import Cc, P, Z
assert re.search(Cc.REGEX, "\n") is not None
assert re.search(Cc.REGEX, "A") is None
assert re.search(P.REGEX, "!") is not None
assert re.search(P.REGEX, "A") is None
assert re.search(Z.REGEX, " ") is not None
assert re.search(Z.REGEX, chr(0x3000)) is not None
assert re.search(Z.REGEX, "A") is None
print("verified uc-micro-py")
