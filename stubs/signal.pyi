"""Limited signal compatibility.

signal/getsignal only store Python handlers; they do not install OS handlers.
alarm returns zero without scheduling anything and pause returns immediately.
raise_signal sends a real process signal through libc; stored Python handlers
are not invoked by that operation. Signal constants retain legacy numeric values.
"""

from typing import Any

SIG_DFL: int
SIG_IGN: int
SIGHUP: int
SIGINT: int
SIGQUIT: int
SIGILL: int
SIGTRAP: int
SIGABRT: int
SIGBUS: int
SIGFPE: int
SIGKILL: int
SIGUSR1: int
SIGSEGV: int
SIGUSR2: int
SIGPIPE: int
SIGALRM: int
SIGTERM: int
SIGCHLD: int
SIGCONT: int
SIGSTOP: int
SIGTSTP: int

def signal(signum: int, handler: Any) -> Any: ...
def getsignal(signum: int) -> Any: ...
def alarm(seconds: int) -> int: ...
def pause() -> None: ...
def raise_signal(signum: int) -> None: ...
