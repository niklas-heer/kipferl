"""
tui - Native UI components module for ucharm
"""

from typing import Any, Callable, Iterable, Iterator, List, Optional, TypeVar

BORDER_ROUNDED: int
BORDER_SQUARE: int
BORDER_DOUBLE: int
BORDER_HEAVY: int
BORDER_NONE: int
ALIGN_LEFT: int
ALIGN_RIGHT: int
ALIGN_CENTER: int

def style(
    text: str,
    *,
    fg: Optional[str] = None,
    bg: Optional[str] = None,
    bold: bool = False,
    dim: bool = False,
    italic: bool = False,
    underline: bool = False,
    strikethrough: bool = False,
) -> str:
    """
    Apply ANSI styling to text and return the styled string.
    """
    ...

def box(
    content: str,
    *,
    title: Optional[str] = None,
    border: str = "rounded",
    border_color: Optional[str] = None,
    padding: int = 1,
) -> None: ...
def rule(
    title: Optional[str] = None,
    char: Optional[str] = None,
    color: Optional[str] = None,
    width: int = 80,
) -> None:
    """
    Print a horizontal rule with optional centered title.
    """
    ...

def progress(
    current: int,
    total: int,
    *,
    label: Optional[str] = None,
    width: int = 40,
    color: Optional[str] = None,
    elapsed: Optional[float] = None,
) -> None:
    """
    Display an animated progress bar with percentage.

    Args:
        current: Current progress value
        total: Total value for 100% completion
        label: Optional label to display before the bar
        width: Width of the progress bar in characters (default 40)
        color: Optional color for the progress bar
        elapsed: Optional elapsed time in seconds to display
    """
    ...

def progress_done() -> None:
    """
    Print a newline after progress/spinner output to complete the line.
    """
    ...

def visible_len(value: Any) -> int:
    """
    Calculate visible length of text, ignoring ANSI escape sequences.
    """
    ...

def success(value: Any) -> None:
    """
    Print a success message with a green checkmark symbol.
    """
    ...

def error(value: Any) -> None:
    """
    Print an error message with a red cross symbol.
    """
    ...

def warning(value: Any) -> None:
    """
    Print a warning message with a yellow warning symbol.
    """
    ...

def info(value: Any) -> None:
    """
    Print an info message with a blue info symbol.
    """
    ...

def spinner_frame(index: int) -> str:
    """
    Get a spinner animation frame by index (cycles through frames).
    """
    ...

def spinner(
    frame: int,
    message: Optional[str] = None,
    color: Optional[str] = None,
) -> None:
    """
    Display a spinner animation frame with optional message.

    Args:
        frame: Frame index (cycles through spinner frames)
        message: Optional message to display after the spinner
        color: Optional color for the spinner
    """
    ...

def table(
    rows: List[List[str]],
    *,
    headers: bool = False,
    border: str = "square",
    border_color: Optional[str] = None,
) -> None:
    """
    Display a formatted table with borders.

    Args:
        rows: List of rows, where each row is a list of cell values
        headers: If True, style the first row as headers with a separator
        border: Border style - "square", "rounded", "double", "heavy", or "none"
        border_color: Optional color for the border

    Example:
        table([["Name", "Age"], ["Alice", "25"]], headers=True)
    """
    ...
