const std = @import("std");
const pk = @import("pk");
const c = pk.c;
const tui_core = @import("tui_core");
const RULE_CHAR = "\xe2\x94\x80";

fn writeOut(bytes: []const u8) void {
    _ = std.posix.write(std.posix.STDOUT_FILENO, bytes) catch {};
}

fn writeCStr(cstr: [*:0]const u8) void {
    writeOut(std.mem.span(cstr));
}

fn appendCode(buf: []u8, pos: *usize, has_codes: *bool, code: []const u8) void {
    if (has_codes.*) {
        buf[pos.*] = ';';
        pos.* += 1;
    }
    @memcpy(buf[pos.* .. pos.* + code.len], code);
    pos.* += code.len;
    has_codes.* = true;
}

fn appendInt(buf: []u8, pos: *usize, has_codes: *bool, value: i32) void {
    if (has_codes.*) {
        buf[pos.*] = ';';
        pos.* += 1;
    }
    const slice = std.fmt.bufPrint(buf[pos.*..], "{d}", .{value}) catch {
        return;
    };
    pos.* += slice.len;
    has_codes.* = true;
}

fn appendRgb(buf: []u8, pos: *usize, has_codes: *bool, prefix: []const u8, r: u8, g: u8, b: u8) void {
    if (has_codes.*) {
        buf[pos.*] = ';';
        pos.* += 1;
    }
    const slice = std.fmt.bufPrint(buf[pos.*..], "{s}{d};{d};{d}", .{ prefix, r, g, b }) catch {
        return;
    };
    pos.* += slice.len;
    has_codes.* = true;
}

fn buildStyleCode(
    buf: []u8,
    fg: ?[*:0]const u8,
    bg: ?[*:0]const u8,
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
) usize {
    var pos: usize = 0;
    var has_codes = false;

    buf[pos] = 0x1b;
    buf[pos + 1] = '[';
    pos += 2;

    if (bold) appendCode(buf, &pos, &has_codes, "1");
    if (dim) appendCode(buf, &pos, &has_codes, "2");
    if (italic) appendCode(buf, &pos, &has_codes, "3");
    if (underline) appendCode(buf, &pos, &has_codes, "4");
    if (strikethrough) appendCode(buf, &pos, &has_codes, "9");

    if (fg) |fg_c| {
        if (fg_c[0] != 0) {
            const code = tui_core.tui_color_code(fg_c);
            if (code >= 0) {
                appendInt(buf, &pos, &has_codes, code);
            } else if (fg_c[0] == '#') {
                var r: u8 = 0;
                var g: u8 = 0;
                var b: u8 = 0;
                if (tui_core.tui_parse_hex(fg_c, &r, &g, &b)) {
                    appendRgb(buf, &pos, &has_codes, "38;2;", r, g, b);
                }
            }
        }
    }

    if (bg) |bg_c| {
        if (bg_c[0] != 0) {
            const code = tui_core.tui_color_code(bg_c);
            if (code >= 0) {
                appendInt(buf, &pos, &has_codes, code + 10);
            } else if (bg_c[0] == '#') {
                var r: u8 = 0;
                var g: u8 = 0;
                var b: u8 = 0;
                if (tui_core.tui_parse_hex(bg_c, &r, &g, &b)) {
                    appendRgb(buf, &pos, &has_codes, "48;2;", r, g, b);
                }
            }
        }
    }

    if (!has_codes) return 0;
    buf[pos] = 'm';
    pos += 1;
    return pos;
}

fn visibleLenSlice(line: []const u8) usize {
    if (line.len == 0) return 0;
    if (line.len < 1024) {
        var tmp: [1024]u8 = undefined;
        @memcpy(tmp[0..line.len], line);
        tmp[line.len] = 0;
        return tui_core.tui_visible_len(@ptrCast(&tmp));
    }
    const buf = std.heap.page_allocator.alloc(u8, line.len + 1) catch {
        return line.len;
    };
    defer std.heap.page_allocator.free(buf);
    @memcpy(buf[0..line.len], line);
    buf[line.len] = 0;
    return tui_core.tui_visible_len(@ptrCast(buf.ptr));
}

fn maxLineVisibleLen(content: []const u8) usize {
    var max_len: usize = 0;
    var it = std.mem.splitScalar(u8, content, '\n');
    while (it.next()) |line| {
        const len = visibleLenSlice(line);
        if (len > max_len) max_len = len;
    }
    return max_len;
}

fn visibleLenFn(ctx: *pk.Context) bool {
    const text = ctx.argStr(0) orelse return ctx.typeError("text must be a string");
    var tmp: [4096]u8 = undefined;
    if (text.len >= tmp.len) {
        return ctx.valueError("text too long");
    }
    @memcpy(tmp[0..text.len], text);
    tmp[text.len] = 0;
    const len = tui_core.tui_visible_len(@ptrCast(&tmp));
    return ctx.returnInt(@intCast(len));
}

fn styleFn(ctx: *pk.Context) bool {
    const text = ctx.argStr(0) orelse return ctx.typeError("text must be a string");

    // Get optional arguments
    var fg_arg = ctx.arg(1);
    var bg_arg = ctx.arg(2);
    const bold = ctx.argBool(3) orelse false;
    const dim = ctx.argBool(4) orelse false;
    const italic = ctx.argBool(5) orelse false;
    const underline = ctx.argBool(6) orelse false;
    const strikethrough = ctx.argBool(7) orelse false;

    const fg: ?[*:0]const u8 = if (fg_arg != null and !fg_arg.?.isNone()) blk: {
        const s = fg_arg.?.toStr() orelse break :blk null;
        break :blk @ptrCast(s.ptr);
    } else null;

    const bg: ?[*:0]const u8 = if (bg_arg != null and !bg_arg.?.isNone()) blk: {
        const s = bg_arg.?.toStr() orelse break :blk null;
        break :blk @ptrCast(s.ptr);
    } else null;

    var style_buf: [128]u8 = undefined;
    const style_len = buildStyleCode(&style_buf, fg, bg, bold, dim, italic, underline, strikethrough);
    if (style_len == 0) {
        return ctx.returnStr(text);
    }

    const total_len = style_len + text.len + 4;
    const out = c.py_newstrn(c.py_retval(), @intCast(total_len));
    const out_slice = @as([*]u8, @ptrCast(out))[0..total_len];
    @memcpy(out_slice[0..style_len], style_buf[0..style_len]);
    @memcpy(out_slice[style_len .. style_len + text.len], text);
    @memcpy(out_slice[style_len + text.len .. total_len], "\x1b[0m");
    return true;
}

fn boxFn(ctx: *pk.Context) bool {
    const content = ctx.argStr(0) orelse return ctx.typeError("content must be a string");

    var title_arg = ctx.arg(1);
    var border_arg = ctx.arg(2);
    var border_color_arg = ctx.arg(3);
    const padding: u32 = @intCast(ctx.argInt(4) orelse 1);

    const title_c: ?[*:0]const u8 = if (title_arg != null and !title_arg.?.isNone()) blk: {
        const s = title_arg.?.toStr() orelse break :blk null;
        break :blk @ptrCast(s.ptr);
    } else null;

    const border_c: [*:0]const u8 = if (border_arg != null and !border_arg.?.isNone()) blk: {
        const s = border_arg.?.toStr() orelse break :blk "rounded";
        break :blk @ptrCast(s.ptr);
    } else "rounded";

    const border_color_c: ?[*:0]const u8 = if (border_color_arg != null and !border_color_arg.?.isNone()) blk: {
        const s = border_color_arg.?.toStr() orelse break :blk null;
        break :blk @ptrCast(s.ptr);
    } else null;

    var border_style: u8 = tui_core.BORDER_ROUNDED;
    const border = std.mem.span(border_c);
    if (std.mem.eql(u8, border, "square")) {
        border_style = tui_core.BORDER_SQUARE;
    } else if (std.mem.eql(u8, border, "double")) {
        border_style = tui_core.BORDER_DOUBLE;
    } else if (std.mem.eql(u8, border, "heavy")) {
        border_style = tui_core.BORDER_HEAVY;
    } else if (std.mem.eql(u8, border, "none")) {
        border_style = tui_core.BORDER_NONE;
    }

    const tl = std.mem.span(tui_core.tui_box_char(border_style, 0));
    const tr = std.mem.span(tui_core.tui_box_char(border_style, 1));
    const bl = std.mem.span(tui_core.tui_box_char(border_style, 2));
    const br = std.mem.span(tui_core.tui_box_char(border_style, 3));
    const h = std.mem.span(tui_core.tui_box_char(border_style, 4));
    const v = std.mem.span(tui_core.tui_box_char(border_style, 5));

    const max_width = maxLineVisibleLen(content);
    const title_len = if (title_c != null) std.mem.span(title_c.?).len else 0;
    const title_width = if (title_c != null) title_len + 4 else 0;
    var content_width = max_width;
    if (title_width > 2 and title_width - 2 > content_width) {
        content_width = title_width - 2;
    }
    const inner_width: usize = content_width + @as(usize, padding) * 2;

    var color_start_buf: [64]u8 = undefined;
    var color_start_len: usize = 0;
    var color_end: []const u8 = "";
    if (border_color_c != null) {
        color_start_len = buildStyleCode(&color_start_buf, border_color_c.?, null, false, false, false, false, false);
        if (color_start_len > 0) color_end = "\x1b[0m";
    }
    const color_start = color_start_buf[0..color_start_len];

    var repeat_buf: [512]u8 = undefined;
    if (title_c != null) {
        writeOut(color_start);
        writeOut(tl);
        writeOut(h);
        writeOut(color_end);
        writeOut("\x1b[1m ");
        writeCStr(title_c.?);
        writeOut(" \x1b[0m");

        const remaining = inner_width - title_len - 3;
        const rep_len = tui_core.tui_repeat(@ptrCast(h.ptr), @intCast(remaining), &repeat_buf);
        writeOut(color_start);
        writeOut(repeat_buf[0..rep_len]);
        writeOut(tr);
        writeOut(color_end);
        writeOut("\n");
    } else {
        const rep_len = tui_core.tui_repeat(@ptrCast(h.ptr), @intCast(inner_width), &repeat_buf);
        writeOut(color_start);
        writeOut(tl);
        writeOut(repeat_buf[0..rep_len]);
        writeOut(tr);
        writeOut(color_end);
        writeOut("\n");
    }

    var pad_spaces: [64]u8 = undefined;
    const pad_len = tui_core.tui_repeat(" ", padding, &pad_spaces);
    const pad_slice = pad_spaces[0..pad_len];

    var it = std.mem.splitScalar(u8, content, '\n');
    var any_line = false;
    while (it.next()) |line| {
        any_line = true;
        const line_buf = std.heap.page_allocator.alloc(u8, line.len + 1) catch {
            return ctx.runtimeError("out of memory");
        };
        defer std.heap.page_allocator.free(line_buf);
        @memcpy(line_buf[0..line.len], line);
        line_buf[line.len] = 0;

        const pad_buf = std.heap.page_allocator.alloc(u8, line.len + inner_width + 1) catch {
            return ctx.runtimeError("out of memory");
        };
        defer std.heap.page_allocator.free(pad_buf);
        const pad_len_line = tui_core.tui_pad(@ptrCast(line_buf.ptr), @intCast(content_width), 0, pad_buf.ptr);

        writeOut(color_start);
        writeOut(v);
        writeOut(color_end);
        writeOut(pad_slice);
        writeOut(pad_buf[0..pad_len_line]);
        writeOut(pad_slice);
        writeOut(color_start);
        writeOut(v);
        writeOut(color_end);
        writeOut("\n");
    }

    if (!any_line) {
        const pad_buf = std.heap.page_allocator.alloc(u8, inner_width + 1) catch {
            return ctx.runtimeError("out of memory");
        };
        defer std.heap.page_allocator.free(pad_buf);
        const pad_len_line = tui_core.tui_pad("", @intCast(content_width), 0, pad_buf.ptr);

        writeOut(color_start);
        writeOut(v);
        writeOut(color_end);
        writeOut(pad_slice);
        writeOut(pad_buf[0..pad_len_line]);
        writeOut(pad_slice);
        writeOut(color_start);
        writeOut(v);
        writeOut(color_end);
        writeOut("\n");
    }

    const rep_len = tui_core.tui_repeat(@ptrCast(h.ptr), @intCast(inner_width), &repeat_buf);
    writeOut(color_start);
    writeOut(bl);
    writeOut(repeat_buf[0..rep_len]);
    writeOut(br);
    writeOut(color_end);
    writeOut("\n");

    return ctx.returnNone();
}

fn ruleFn(ctx: *pk.Context) bool {
    var title_arg = ctx.arg(0);
    var char_arg = ctx.arg(1);
    var color_arg = ctx.arg(2);
    const width: i32 = @intCast(ctx.argInt(3) orelse 80);

    const title_c: ?[*:0]const u8 = if (title_arg != null and !title_arg.?.isNone()) blk: {
        const s = title_arg.?.toStr() orelse break :blk null;
        break :blk @ptrCast(s.ptr);
    } else null;

    const ch_c: [*:0]const u8 = if (char_arg != null and !char_arg.?.isNone()) blk: {
        const s = char_arg.?.toStr() orelse break :blk RULE_CHAR;
        break :blk @ptrCast(s.ptr);
    } else RULE_CHAR;

    const color_c: ?[*:0]const u8 = if (color_arg != null and !color_arg.?.isNone()) blk: {
        const s = color_arg.?.toStr() orelse break :blk null;
        break :blk @ptrCast(s.ptr);
    } else null;

    var color_start_buf: [64]u8 = undefined;
    var color_start_len: usize = 0;
    var color_end: []const u8 = "";
    if (color_c != null) {
        color_start_len = buildStyleCode(&color_start_buf, color_c.?, null, false, false, false, false, false);
        if (color_start_len > 0) color_end = "\x1b[0m";
    }
    const color_start = color_start_buf[0..color_start_len];

    var repeat_buf: [512]u8 = undefined;
    if (title_c != null) {
        const title_len = std.mem.span(title_c.?).len;
        var side: i32 = @divTrunc(width - @as(i32, @intCast(title_len)) - 2, 2);
        if (side < 0) side = 0;
        const rep_len = tui_core.tui_repeat(ch_c, @intCast(side), &repeat_buf);
        writeOut(color_start);
        writeOut(repeat_buf[0..rep_len]);
        writeOut(color_end);
        writeOut(" ");
        writeCStr(title_c.?);
        writeOut(" ");
        var remaining: i32 = width - side - @as(i32, @intCast(title_len)) - 2;
        if (remaining < 0) remaining = 0;
        const rep_len2 = tui_core.tui_repeat(ch_c, @intCast(remaining), &repeat_buf);
        writeOut(color_start);
        writeOut(repeat_buf[0..rep_len2]);
        writeOut(color_end);
        writeOut("\n");
    } else {
        const rep_len = tui_core.tui_repeat(ch_c, @intCast(width), &repeat_buf);
        writeOut(color_start);
        writeOut(repeat_buf[0..rep_len]);
        writeOut(color_end);
        writeOut("\n");
    }

    return ctx.returnNone();
}

fn successFn(ctx: *pk.Context) bool {
    const msg = ctx.argStr(0) orelse return ctx.typeError("message must be a string");
    writeOut("\x1b[1;32m");
    writeCStr(tui_core.tui_symbol_success());
    writeOut(" \x1b[0m");
    writeOut(msg);
    writeOut("\n");
    return ctx.returnNone();
}

fn errorMsgFn(ctx: *pk.Context) bool {
    const msg = ctx.argStr(0) orelse return ctx.typeError("message must be a string");
    writeOut("\x1b[1;31m");
    writeCStr(tui_core.tui_symbol_error());
    writeOut(" \x1b[0m");
    writeOut(msg);
    writeOut("\n");
    return ctx.returnNone();
}

fn warningFn(ctx: *pk.Context) bool {
    const msg = ctx.argStr(0) orelse return ctx.typeError("message must be a string");
    writeOut("\x1b[1;33m");
    writeCStr(tui_core.tui_symbol_warning());
    writeOut(" \x1b[0m");
    writeOut(msg);
    writeOut("\n");
    return ctx.returnNone();
}

fn infoFn(ctx: *pk.Context) bool {
    const msg = ctx.argStr(0) orelse return ctx.typeError("message must be a string");
    writeOut("\x1b[1;34m");
    writeCStr(tui_core.tui_symbol_info());
    writeOut(" \x1b[0m");
    writeOut(msg);
    writeOut("\n");
    return ctx.returnNone();
}

fn progressFn(ctx: *pk.Context) bool {
    const current: u32 = @intCast(ctx.argInt(0) orelse return ctx.typeError("current must be int"));
    const total: u32 = @intCast(ctx.argInt(1) orelse return ctx.typeError("total must be int"));

    var label_arg = ctx.arg(2);
    const width: u32 = @intCast(ctx.argInt(3) orelse 40);
    var color_arg = ctx.arg(4);
    var elapsed_arg = ctx.arg(5);

    const label_c: ?[*:0]const u8 = if (label_arg != null and !label_arg.?.isNone()) blk: {
        const s = label_arg.?.toStr() orelse break :blk null;
        break :blk @ptrCast(s.ptr);
    } else null;

    const color_c: ?[*:0]const u8 = if (color_arg != null and !color_arg.?.isNone()) blk: {
        const s = color_arg.?.toStr() orelse break :blk null;
        break :blk @ptrCast(s.ptr);
    } else null;

    // Get elapsed time (optional float in seconds)
    const elapsed: ?f64 = if (elapsed_arg != null and !elapsed_arg.?.isNone()) blk: {
        break :blk elapsed_arg.?.toNumber();
    } else null;

    var bar_buf: [256]u8 = undefined;
    var percent_buf: [16]u8 = undefined;
    const bar_len = tui_core.tui_progress_bar(current, total, width, &bar_buf);
    const percent_len = tui_core.tui_percent_str(current, total, &percent_buf);

    var color_start_buf: [64]u8 = undefined;
    var color_start_len: usize = 0;
    var color_end: []const u8 = "";
    if (color_c != null) {
        color_start_len = buildStyleCode(&color_start_buf, color_c.?, null, false, false, false, false, false);
        if (color_start_len > 0) color_end = "\x1b[0m";
    }
    const color_start = color_start_buf[0..color_start_len];

    writeOut("\r");
    if (label_c != null) {
        writeCStr(label_c.?);
        writeOut(" ");
    }
    writeOut(color_start);
    writeOut(bar_buf[0..bar_len]);
    writeOut(color_end);
    writeOut(" ");
    writeOut(percent_buf[0..percent_len]);

    // Write elapsed time if provided
    if (elapsed) |e| {
        var time_buf: [32]u8 = undefined;
        const time_str = std.fmt.bufPrint(&time_buf, "  {d:.1}s", .{e}) catch "";
        writeOut(time_str);
    }

    // Clear rest of line (in case previous output was longer)
    writeOut("\x1b[K");

    return ctx.returnNone();
}

fn spinnerFrameFn(ctx: *pk.Context) bool {
    const index: u32 = @intCast(ctx.argInt(0) orelse return ctx.typeError("index must be int"));
    c.py_newstr(c.py_retval(), tui_core.tui_spinner_frame(index));
    return true;
}

fn spinnerFn(ctx: *pk.Context) bool {
    // arg 0: frame index (int)
    // arg 1: message (optional str)
    // arg 2: color (optional str)
    const index: u32 = @intCast(ctx.argInt(0) orelse return ctx.typeError("index must be int"));

    var msg_arg = ctx.arg(1);
    var color_arg = ctx.arg(2);

    const msg: ?[]const u8 = if (msg_arg != null and !msg_arg.?.isNone()) blk: {
        break :blk msg_arg.?.toStr();
    } else null;

    const color_c: ?[*:0]const u8 = if (color_arg != null and !color_arg.?.isNone()) blk: {
        const s = color_arg.?.toStr() orelse break :blk null;
        break :blk @ptrCast(s.ptr);
    } else null;

    var color_start_buf: [64]u8 = undefined;
    var color_start_len: usize = 0;
    var color_end: []const u8 = "";
    if (color_c != null) {
        color_start_len = buildStyleCode(&color_start_buf, color_c.?, null, false, false, false, false, false);
        if (color_start_len > 0) color_end = "\x1b[0m";
    }
    const color_start = color_start_buf[0..color_start_len];

    writeOut("\r");
    writeOut(color_start);
    writeCStr(tui_core.tui_spinner_frame(index));
    writeOut(color_end);
    if (msg) |m| {
        writeOut(" ");
        writeOut(m);
    }
    writeOut("\x1b[K"); // Clear to end of line

    return ctx.returnNone();
}

fn progressDoneFn(ctx: *pk.Context) bool {
    // Just print a newline to finish progress/spinner output
    writeOut("\n");
    return ctx.returnNone();
}

// Table character position constants
const TC_H: u8 = 0; // horizontal
const TC_V: u8 = 1; // vertical
const TC_TL: u8 = 2; // top-left
const TC_TR: u8 = 3; // top-right
const TC_BL: u8 = 4; // bottom-left
const TC_BR: u8 = 5; // bottom-right
const TC_TH: u8 = 6; // top horizontal junction
const TC_BH: u8 = 7; // bottom horizontal junction
const TC_LV: u8 = 8; // left vertical junction
const TC_RV: u8 = 9; // right vertical junction
const TC_CROSS: u8 = 10; // cross junction

fn tableFn(ctx: *pk.Context) bool {
    // arg 0: rows (list of lists)
    // arg 1: headers (bool, default False) - first row is header
    // arg 2: border (str, default "square")
    // arg 3: border_color (str, optional)

    var rows_val = ctx.arg(0) orelse return ctx.typeError("rows must be a list");
    if (!rows_val.isList()) return ctx.typeError("rows must be a list");

    var border_arg = ctx.arg(2);
    var border_color_arg = ctx.arg(3);
    const has_headers = ctx.argBool(1) orelse false;

    // Determine border style
    var border_style: u8 = tui_core.BORDER_SQUARE;
    if (border_arg != null and !border_arg.?.isNone()) {
        if (border_arg.?.toStr()) |border_str| {
            if (std.mem.eql(u8, border_str, "rounded")) {
                border_style = tui_core.BORDER_ROUNDED;
            } else if (std.mem.eql(u8, border_str, "double")) {
                border_style = tui_core.BORDER_DOUBLE;
            } else if (std.mem.eql(u8, border_str, "heavy")) {
                border_style = tui_core.BORDER_HEAVY;
            } else if (std.mem.eql(u8, border_str, "none")) {
                border_style = tui_core.BORDER_NONE;
            }
        }
    }

    // Get border color
    const border_color_c: ?[*:0]const u8 = if (border_color_arg != null and !border_color_arg.?.isNone()) blk: {
        const s = border_color_arg.?.toStr() orelse break :blk null;
        break :blk @ptrCast(s.ptr);
    } else null;

    var color_start_buf: [64]u8 = undefined;
    var color_start_len: usize = 0;
    var color_end: []const u8 = "";
    if (border_color_c != null) {
        color_start_len = buildStyleCode(&color_start_buf, border_color_c.?, null, false, false, false, false, false);
        if (color_start_len > 0) color_end = "\x1b[0m";
    }
    const color_start = color_start_buf[0..color_start_len];

    // Get table characters
    const h = std.mem.span(tui_core.tui_table_char(border_style, TC_H));
    const v = std.mem.span(tui_core.tui_table_char(border_style, TC_V));
    const tl = std.mem.span(tui_core.tui_table_char(border_style, TC_TL));
    const tr = std.mem.span(tui_core.tui_table_char(border_style, TC_TR));
    const bl = std.mem.span(tui_core.tui_table_char(border_style, TC_BL));
    const br = std.mem.span(tui_core.tui_table_char(border_style, TC_BR));
    const th = std.mem.span(tui_core.tui_table_char(border_style, TC_TH));
    const bh = std.mem.span(tui_core.tui_table_char(border_style, TC_BH));
    const lv = std.mem.span(tui_core.tui_table_char(border_style, TC_LV));
    const rv = std.mem.span(tui_core.tui_table_char(border_style, TC_RV));
    const cross = std.mem.span(tui_core.tui_table_char(border_style, TC_CROSS));

    // Get row count - use Value.len() which works for lists
    const num_rows: usize = rows_val.len() orelse return ctx.typeError("rows must be a list");
    if (num_rows == 0) return ctx.returnNone();

    // Calculate column count from first row
    var first_row = rows_val.getItem(0) orelse return ctx.typeError("rows must not be empty");
    if (!first_row.isList()) return ctx.typeError("each row must be a list");
    const num_cols: usize = first_row.len() orelse 0;
    if (num_cols == 0) return ctx.returnNone();

    // Calculate column widths (visible width)
    const max_cols = 32;
    var col_widths: [max_cols]usize = [_]usize{0} ** max_cols;
    const actual_cols = @min(num_cols, max_cols);

    for (0..num_rows) |row_idx| {
        var row = rows_val.getItem(row_idx) orelse continue;
        if (!row.isList()) continue;
        const row_len: usize = row.len() orelse 0;
        for (0..@min(row_len, actual_cols)) |col_idx| {
            var cell = row.getItem(col_idx) orelse continue;
            const cell_str = cell.toStr() orelse "";
            const vis_len = visibleLenSlice(cell_str);
            if (vis_len > col_widths[col_idx]) {
                col_widths[col_idx] = vis_len;
            }
        }
    }

    var repeat_buf: [512]u8 = undefined;

    // Helper to write horizontal line
    const writeHorizLine = struct {
        fn call(
            left: []const u8,
            mid: []const u8,
            right: []const u8,
            horiz: []const u8,
            widths: []usize,
            cols: usize,
            cstart: []const u8,
            cend: []const u8,
            rbuf: *[512]u8,
        ) void {
            writeOut(cstart);
            writeOut(left);
            for (0..cols) |i| {
                const rep_len = tui_core.tui_repeat(@ptrCast(horiz.ptr), @intCast(widths[i] + 2), rbuf);
                writeOut(rbuf[0..rep_len]);
                if (i < cols - 1) {
                    writeOut(mid);
                }
            }
            writeOut(right);
            writeOut(cend);
            writeOut("\n");
        }
    }.call;

    // Write top border
    writeHorizLine(tl, th, tr, h, &col_widths, actual_cols, color_start, color_end, &repeat_buf);

    // Write each row
    for (0..num_rows) |row_idx| {
        var row = rows_val.getItem(row_idx) orelse continue;
        if (!row.isList()) continue;
        const row_len: usize = row.len() orelse 0;

        // Write row content
        writeOut(color_start);
        writeOut(v);
        writeOut(color_end);

        for (0..actual_cols) |col_idx| {
            writeOut(" ");
            var cell_str: []const u8 = "";
            if (col_idx < row_len) {
                var cell = row.getItem(col_idx) orelse continue;
                cell_str = cell.toStr() orelse "";
            }

            // Apply bold for header row
            if (has_headers and row_idx == 0) {
                writeOut("\x1b[1m");
            }

            // Write cell content
            writeOut(cell_str);

            if (has_headers and row_idx == 0) {
                writeOut("\x1b[0m");
            }

            // Pad to column width
            const vis_len = visibleLenSlice(cell_str);
            const pad_needed = col_widths[col_idx] - vis_len;
            for (0..pad_needed) |_| {
                writeOut(" ");
            }
            writeOut(" ");

            writeOut(color_start);
            writeOut(v);
            writeOut(color_end);
        }
        writeOut("\n");

        // Write separator after header row
        if (has_headers and row_idx == 0 and num_rows > 1) {
            writeHorizLine(lv, cross, rv, h, &col_widths, actual_cols, color_start, color_end, &repeat_buf);
        }
    }

    // Write bottom border
    writeHorizLine(bl, bh, br, h, &col_widths, actual_cols, color_start, color_end, &repeat_buf);

    return ctx.returnNone();
}

pub fn register() void {
    var builder = pk.ModuleBuilder.new("tui");
    _ = builder
        .funcWrapped("visible_len", 1, 1, visibleLenFn)
        // Use signature-based binding for kwargs support
        .funcSigWrapped("style(text, fg=None, bg=None, bold=False, dim=False, italic=False, underline=False, strikethrough=False)", 1, 8, styleFn)
        .funcSigWrapped("box(content, title=None, border_color=None, padding=0, border_style=None)", 1, 5, boxFn)
        .funcSigWrapped("rule(title=None, color=None, align=0, width=0)", 0, 4, ruleFn)
        .funcWrapped("success", 1, 1, successFn)
        .funcWrapped("error", 1, 1, errorMsgFn)
        .funcWrapped("warning", 1, 1, warningFn)
        .funcWrapped("info", 1, 1, infoFn)
        .funcSigWrapped("progress(current, total, label=None, width=40, color=None, elapsed=None)", 2, 6, progressFn)
        .funcWrapped("progress_done", 0, 0, progressDoneFn)
        .funcWrapped("spinner_frame", 1, 1, spinnerFrameFn)
        .funcSigWrapped("spinner(frame, message=None, color=None)", 1, 3, spinnerFn)
        .funcSigWrapped("table(rows, headers=False, border='square', border_color=None)", 1, 4, tableFn)
        .constInt("BORDER_ROUNDED", tui_core.BORDER_ROUNDED)
        .constInt("BORDER_SQUARE", tui_core.BORDER_SQUARE)
        .constInt("BORDER_DOUBLE", tui_core.BORDER_DOUBLE)
        .constInt("BORDER_HEAVY", tui_core.BORDER_HEAVY)
        .constInt("BORDER_NONE", tui_core.BORDER_NONE)
        .constInt("ALIGN_LEFT", 0)
        .constInt("ALIGN_RIGHT", 1)
        .constInt("ALIGN_CENTER", 2);
}
