// tui.zig - UI display components for ucharm
//
// This module provides high-level UI rendering:
// - Box rendering with multiple border styles
// - Rule/divider rendering
// - Status messages (success, error, warning, info)
// - Progress bar rendering
// - ANSI style helpers

const std = @import("std");

// ============================================================================
// Types
// ============================================================================

pub const CStr = [*:0]const u8;

// Border style constants
pub const BORDER_ROUNDED: u8 = 0;
pub const BORDER_SQUARE: u8 = 1;
pub const BORDER_DOUBLE: u8 = 2;
pub const BORDER_HEAVY: u8 = 3;
pub const BORDER_NONE: u8 = 4;

// Box characters structure
const BoxChars = struct {
    tl: []const u8,
    tr: []const u8,
    bl: []const u8,
    br: []const u8,
    h: []const u8,
    v: []const u8,
};

const box_chars = [5]BoxChars{
    // rounded
    .{ .tl = "╭", .tr = "╮", .bl = "╰", .br = "╯", .h = "─", .v = "│" },
    // square
    .{ .tl = "┌", .tr = "┐", .bl = "└", .br = "┘", .h = "─", .v = "│" },
    // double
    .{ .tl = "╔", .tr = "╗", .bl = "╚", .br = "╝", .h = "═", .v = "║" },
    // heavy
    .{ .tl = "┏", .tr = "┓", .bl = "┗", .br = "┛", .h = "━", .v = "┃" },
    // none
    .{ .tl = " ", .tr = " ", .bl = " ", .br = " ", .h = " ", .v = " " },
};

// Status symbols (UTF-8)
const SYMBOL_SUCCESS = "✓";
const SYMBOL_ERROR = "✗";
const SYMBOL_WARNING = "⚠";
const SYMBOL_INFO = "ℹ";
const SYMBOL_BULLET = "•";

// Progress bar characters
const PROGRESS_FILL = "█";
const PROGRESS_EMPTY = "░";

// Table junction characters (for square/light box drawing)
const TableChars = struct {
    h: []const u8, // horizontal
    v: []const u8, // vertical
    tl: []const u8, // top-left corner
    tr: []const u8, // top-right corner
    bl: []const u8, // bottom-left corner
    br: []const u8, // bottom-right corner
    th: []const u8, // top horizontal junction (T pointing down)
    bh: []const u8, // bottom horizontal junction (T pointing up)
    lv: []const u8, // left vertical junction (T pointing right)
    rv: []const u8, // right vertical junction (T pointing left)
    cross: []const u8, // cross junction
};

const table_chars = [5]TableChars{
    // rounded (use square junctions, rounded corners)
    .{ .h = "─", .v = "│", .tl = "╭", .tr = "╮", .bl = "╰", .br = "╯", .th = "┬", .bh = "┴", .lv = "├", .rv = "┤", .cross = "┼" },
    // square
    .{ .h = "─", .v = "│", .tl = "┌", .tr = "┐", .bl = "└", .br = "┘", .th = "┬", .bh = "┴", .lv = "├", .rv = "┤", .cross = "┼" },
    // double
    .{ .h = "═", .v = "║", .tl = "╔", .tr = "╗", .bl = "╚", .br = "╝", .th = "╦", .bh = "╩", .lv = "╠", .rv = "╣", .cross = "╬" },
    // heavy
    .{ .h = "━", .v = "┃", .tl = "┏", .tr = "┓", .bl = "┗", .br = "┛", .th = "┳", .bh = "┻", .lv = "┣", .rv = "┫", .cross = "╋" },
    // none (spaces)
    .{ .h = " ", .v = " ", .tl = " ", .tr = " ", .bl = " ", .br = " ", .th = " ", .bh = " ", .lv = " ", .rv = " ", .cross = " " },
};

// Spinner frames
const SPINNER_FRAMES = [_][]const u8{ "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏" };

// Color codes for standard colors
const ColorCode = struct {
    name: []const u8,
    fg: u8,
};

const standard_colors = [_]ColorCode{
    .{ .name = "black", .fg = 30 },
    .{ .name = "red", .fg = 31 },
    .{ .name = "green", .fg = 32 },
    .{ .name = "yellow", .fg = 33 },
    .{ .name = "blue", .fg = 34 },
    .{ .name = "magenta", .fg = 35 },
    .{ .name = "cyan", .fg = 36 },
    .{ .name = "white", .fg = 37 },
    .{ .name = "gray", .fg = 90 },
    .{ .name = "grey", .fg = 90 },
};

// ============================================================================
// Helper Functions
// ============================================================================

fn cstr_len(s: CStr) usize {
    var i: usize = 0;
    while (s[i] != 0) : (i += 1) {}
    return i;
}

fn streq_slice(cstr: CStr, slice: []const u8) bool {
    for (slice, 0..) |c, i| {
        if (cstr[i] == 0 or cstr[i] != c) return false;
    }
    return cstr[slice.len] == 0;
}

fn write_int(buf: [*]u8, val: u32) usize {
    var pos: usize = 0;
    if (val >= 100) {
        buf[pos] = '0' + @as(u8, @intCast((val / 100) % 10));
        pos += 1;
    }
    if (val >= 10) {
        buf[pos] = '0' + @as(u8, @intCast((val / 10) % 10));
        pos += 1;
    }
    buf[pos] = '0' + @as(u8, @intCast(val % 10));
    pos += 1;
    return pos;
}

fn hex_digit(c: u8) ?u8 {
    if (c >= '0' and c <= '9') return c - '0';
    if (c >= 'a' and c <= 'f') return c - 'a' + 10;
    if (c >= 'A' and c <= 'F') return c - 'A' + 10;
    return null;
}

// ============================================================================
// Exported Functions
// ============================================================================

/// Get visible length of string (excluding ANSI escape codes)
pub export fn tui_visible_len(s: CStr) usize {
    var i: usize = 0;
    var length: usize = 0;

    while (s[i] != 0) {
        if (s[i] == 0x1b and s[i + 1] == '[') {
            i += 2;
            while (s[i] != 0 and s[i] != 'm' and s[i] != 'H' and s[i] != 'J' and s[i] != 'K') {
                i += 1;
            }
            if (s[i] != 0) i += 1;
        } else {
            const c = s[i];
            if (c < 128) {
                length += 1;
                i += 1;
            } else if (c < 0xE0) {
                length += 1;
                i += 2;
            } else if (c < 0xF0) {
                length += 2;
                i += 3;
            } else {
                length += 2;
                i += 4;
            }
        }
    }

    return length;
}

/// Get box character for style and position
/// position: 0=tl, 1=tr, 2=bl, 3=br, 4=h, 5=v
pub export fn tui_box_char(style: u8, position: u8) CStr {
    const idx = if (style < 5) style else 0;
    const chars = &box_chars[idx];
    return switch (position) {
        0 => @ptrCast(chars.tl.ptr),
        1 => @ptrCast(chars.tr.ptr),
        2 => @ptrCast(chars.bl.ptr),
        3 => @ptrCast(chars.br.ptr),
        4 => @ptrCast(chars.h.ptr),
        5 => @ptrCast(chars.v.ptr),
        else => @ptrCast(chars.h.ptr),
    };
}

/// Get success symbol
pub export fn tui_symbol_success() CStr {
    return @ptrCast(SYMBOL_SUCCESS.ptr);
}

/// Get error symbol
pub export fn tui_symbol_error() CStr {
    return @ptrCast(SYMBOL_ERROR.ptr);
}

/// Get warning symbol
pub export fn tui_symbol_warning() CStr {
    return @ptrCast(SYMBOL_WARNING.ptr);
}

/// Get info symbol
pub export fn tui_symbol_info() CStr {
    return @ptrCast(SYMBOL_INFO.ptr);
}

/// Get bullet symbol
pub export fn tui_symbol_bullet() CStr {
    return @ptrCast(SYMBOL_BULLET.ptr);
}

/// Get spinner frame
pub export fn tui_spinner_frame(index: u32) CStr {
    const idx = index % SPINNER_FRAMES.len;
    return @ptrCast(SPINNER_FRAMES[idx].ptr);
}

/// Get spinner frame count
pub export fn tui_spinner_frame_count() u32 {
    return SPINNER_FRAMES.len;
}

/// Get table character for style and position
/// position: 0=h, 1=v, 2=tl, 3=tr, 4=bl, 5=br, 6=th, 7=bh, 8=lv, 9=rv, 10=cross
pub export fn tui_table_char(style: u8, position: u8) CStr {
    const idx = if (style < 5) style else 0;
    const chars = &table_chars[idx];
    return switch (position) {
        0 => @ptrCast(chars.h.ptr),
        1 => @ptrCast(chars.v.ptr),
        2 => @ptrCast(chars.tl.ptr),
        3 => @ptrCast(chars.tr.ptr),
        4 => @ptrCast(chars.bl.ptr),
        5 => @ptrCast(chars.br.ptr),
        6 => @ptrCast(chars.th.ptr),
        7 => @ptrCast(chars.bh.ptr),
        8 => @ptrCast(chars.lv.ptr),
        9 => @ptrCast(chars.rv.ptr),
        10 => @ptrCast(chars.cross.ptr),
        else => @ptrCast(chars.h.ptr),
    };
}

/// Build a progress bar string into the provided buffer
pub export fn tui_progress_bar(current: u32, total: u32, width: u32, buf: [*]u8) usize {
    if (total == 0 or width == 0) return 0;

    const filled_count: u32 = @min(width, (width * current) / total);
    var pos: usize = 0;

    var i: u32 = 0;
    while (i < filled_count) : (i += 1) {
        for (PROGRESS_FILL) |c| {
            buf[pos] = c;
            pos += 1;
        }
    }

    while (i < width) : (i += 1) {
        for (PROGRESS_EMPTY) |c| {
            buf[pos] = c;
            pos += 1;
        }
    }

    buf[pos] = 0;
    return pos;
}

/// Get percentage string
pub export fn tui_percent_str(current: u32, total: u32, buf: [*]u8) usize {
    if (total == 0) {
        buf[0] = '0';
        buf[1] = '%';
        buf[2] = 0;
        return 2;
    }

    const percent: u32 = (100 * current) / total;
    var pos: usize = 0;

    pos += write_int(buf, percent);
    buf[pos] = '%';
    pos += 1;
    buf[pos] = 0;

    return pos;
}

/// Look up color name and return foreground code
pub export fn tui_color_code(name: CStr) i32 {
    for (standard_colors) |color| {
        if (streq_slice(name, color.name)) {
            return @intCast(color.fg);
        }
    }
    return -1;
}

/// Parse hex color string (#RRGGBB) into r,g,b values
pub export fn tui_parse_hex(hex: CStr, out_r: *u8, out_g: *u8, out_b: *u8) bool {
    if (hex[0] != '#') return false;

    var len: usize = 0;
    while (hex[len + 1] != 0) : (len += 1) {}

    if (len == 6) {
        const r1 = hex_digit(hex[1]) orelse return false;
        const r2 = hex_digit(hex[2]) orelse return false;
        const g1 = hex_digit(hex[3]) orelse return false;
        const g2 = hex_digit(hex[4]) orelse return false;
        const b1 = hex_digit(hex[5]) orelse return false;
        const b2 = hex_digit(hex[6]) orelse return false;

        out_r.* = r1 * 16 + r2;
        out_g.* = g1 * 16 + g2;
        out_b.* = b1 * 16 + b2;
        return true;
    } else if (len == 3) {
        const r = hex_digit(hex[1]) orelse return false;
        const g = hex_digit(hex[2]) orelse return false;
        const b = hex_digit(hex[3]) orelse return false;

        out_r.* = r * 17;
        out_g.* = g * 17;
        out_b.* = b * 17;
        return true;
    }

    return false;
}

/// Repeat a pattern n times into buffer
pub export fn tui_repeat(pattern: CStr, count: u32, buf: [*]u8) usize {
    const pattern_len = cstr_len(pattern);
    var pos: usize = 0;

    var i: u32 = 0;
    while (i < count) : (i += 1) {
        for (0..pattern_len) |j| {
            buf[pos] = pattern[j];
            pos += 1;
        }
    }

    buf[pos] = 0;
    return pos;
}

/// Pad a string to width (align: 0=left, 1=right, 2=center)
pub export fn tui_pad(text: CStr, width: u32, align_mode: u8, buf: [*]u8) usize {
    const text_len = cstr_len(text);
    const vis_len = tui_visible_len(text);

    if (vis_len >= width) {
        for (0..text_len) |i| {
            buf[i] = text[i];
        }
        buf[text_len] = 0;
        return text_len;
    }

    const pad_needed: u32 = width - @as(u32, @intCast(vis_len));
    var pos: usize = 0;

    if (align_mode == 1) {
        // Right align
        var i: u32 = 0;
        while (i < pad_needed) : (i += 1) {
            buf[pos] = ' ';
            pos += 1;
        }
        for (0..text_len) |j| {
            buf[pos] = text[j];
            pos += 1;
        }
    } else if (align_mode == 2) {
        // Center align
        const left_pad = pad_needed / 2;
        const right_pad = pad_needed - left_pad;

        var i: u32 = 0;
        while (i < left_pad) : (i += 1) {
            buf[pos] = ' ';
            pos += 1;
        }
        for (0..text_len) |j| {
            buf[pos] = text[j];
            pos += 1;
        }
        i = 0;
        while (i < right_pad) : (i += 1) {
            buf[pos] = ' ';
            pos += 1;
        }
    } else {
        // Left align (default)
        for (0..text_len) |j| {
            buf[pos] = text[j];
            pos += 1;
        }
        var i: u32 = 0;
        while (i < pad_needed) : (i += 1) {
            buf[pos] = ' ';
            pos += 1;
        }
    }

    buf[pos] = 0;
    return pos;
}

// ============================================================================
// Tests
// ============================================================================

test "visible_len_ascii" {
    try std.testing.expectEqual(@as(usize, 5), tui_visible_len("hello"));
    try std.testing.expectEqual(@as(usize, 0), tui_visible_len(""));
}

test "visible_len_ansi" {
    try std.testing.expectEqual(@as(usize, 5), tui_visible_len("\x1b[31mhello\x1b[0m"));
}

test "progress_bar" {
    var buf: [256]u8 = undefined;
    const len = tui_progress_bar(5, 10, 10, &buf);
    try std.testing.expect(len > 0);
}

test "color_code" {
    try std.testing.expectEqual(@as(i32, 31), tui_color_code("red"));
    try std.testing.expectEqual(@as(i32, 32), tui_color_code("green"));
    try std.testing.expectEqual(@as(i32, -1), tui_color_code("purple"));
}

test "parse_hex" {
    var r: u8 = 0;
    var g: u8 = 0;
    var b: u8 = 0;

    try std.testing.expect(tui_parse_hex("#ff5500", &r, &g, &b));
    try std.testing.expectEqual(@as(u8, 255), r);
    try std.testing.expectEqual(@as(u8, 85), g);
    try std.testing.expectEqual(@as(u8, 0), b);
}
