const std = @import("std");
const builtin = @import("builtin");
const Trailer = @import("trailer.zig").Trailer;

/// Platform-specific executor for running the embedded MicroPython with Python code.
///
/// On Linux: Uses memfd_create for zero-disk-write execution
/// On macOS: Uses /tmp with content-hash caching for fast subsequent runs
const MAX_PATH_LEN = 4096;

/// Execution context holding paths to extracted files
pub const ExecContext = struct {
    mpy_path: []const u8,
    py_path: []const u8,
    cache_dir: ?[]const u8,

    /// Clean up temporary files if not using cache
    pub fn cleanup(self: *ExecContext, allocator: std.mem.Allocator) void {
        // Only clean up non-cached files
        if (self.cache_dir == null) {
            std.fs.deleteFileAbsolute(self.mpy_path) catch {};
            std.fs.deleteFileAbsolute(self.py_path) catch {};
        }
        allocator.free(self.mpy_path);
        allocator.free(self.py_path);
        if (self.cache_dir) |dir| {
            allocator.free(dir);
        }
    }
};

/// Prepare MicroPython binary and Python code for execution
pub fn prepare(
    allocator: std.mem.Allocator,
    self_file: std.fs.File,
    trailer: Trailer,
    content_hash: [8]u8,
) !ExecContext {
    if (builtin.os.tag == .linux) {
        return prepareLinux(allocator, self_file, trailer);
    } else {
        return prepareMacOS(allocator, self_file, trailer, content_hash);
    }
}

/// Linux: Use memfd_create for in-memory execution
fn prepareLinux(
    allocator: std.mem.Allocator,
    self_file: std.fs.File,
    trailer: Trailer,
) !ExecContext {
    // Create anonymous memory file for MicroPython binary
    const mpy_fd = std.os.linux.memfd_create("micropython", 0);
    if (mpy_fd < 0) {
        // Fallback to /tmp if memfd_create fails (older kernels)
        return prepareFallback(allocator, self_file, trailer);
    }
    const mpy_file = std.fs.File{ .handle = @intCast(mpy_fd) };
    defer mpy_file.close();

    // Read MicroPython binary from self and write to memfd
    try self_file.seekTo(trailer.micropython_offset);
    var mpy_remaining = trailer.micropython_size;
    var buf: [65536]u8 = undefined;
    while (mpy_remaining > 0) {
        const to_read = @min(mpy_remaining, buf.len);
        const bytes_read = try self_file.read(buf[0..to_read]);
        if (bytes_read == 0) break;
        try mpy_file.writeAll(buf[0..bytes_read]);
        mpy_remaining -= bytes_read;
    }

    // Create memfd for Python code
    const py_fd = std.os.linux.memfd_create("app.py", 0);
    if (py_fd < 0) {
        return error.MemfdCreateFailed;
    }
    const py_file = std.fs.File{ .handle = @intCast(py_fd) };
    defer py_file.close();

    // Read Python code from self and write to memfd
    try self_file.seekTo(trailer.python_offset);
    var py_remaining = trailer.python_size;
    while (py_remaining > 0) {
        const to_read = @min(py_remaining, buf.len);
        const bytes_read = try self_file.read(buf[0..to_read]);
        if (bytes_read == 0) break;
        try py_file.writeAll(buf[0..bytes_read]);
        py_remaining -= bytes_read;
    }

    // Create paths from file descriptors
    const mpy_path = try std.fmt.allocPrint(allocator, "/proc/self/fd/{d}", .{mpy_fd});
    const py_path = try std.fmt.allocPrint(allocator, "/proc/self/fd/{d}", .{py_fd});

    return ExecContext{
        .mpy_path = mpy_path,
        .py_path = py_path,
        .cache_dir = null,
    };
}

/// macOS: Use /tmp with content-hash caching
fn prepareMacOS(
    allocator: std.mem.Allocator,
    self_file: std.fs.File,
    trailer: Trailer,
    content_hash: [8]u8,
) !ExecContext {
    // Convert hash to hex string
    var hash_hex: [16]u8 = undefined;
    const hex_chars = "0123456789abcdef";
    for (0..8) |i| {
        hash_hex[i * 2] = hex_chars[content_hash[i] >> 4];
        hash_hex[i * 2 + 1] = hex_chars[content_hash[i] & 0x0f];
    }

    // Build cache directory path: /tmp/ucharm-{hash}
    const cache_dir = try std.fmt.allocPrint(allocator, "/tmp/ucharm-{s}", .{hash_hex});
    const mpy_path = try std.fmt.allocPrint(allocator, "{s}/m", .{cache_dir});
    const py_path = try std.fmt.allocPrint(allocator, "{s}/a.py", .{cache_dir});

    // Check if cache already exists and is valid
    const cache_valid = blk: {
        std.fs.accessAbsolute(mpy_path, .{}) catch break :blk false;
        std.fs.accessAbsolute(py_path, .{}) catch break :blk false;
        break :blk true;
    };

    if (cache_valid) {
        // Cache hit - use existing files
        return ExecContext{
            .mpy_path = mpy_path,
            .py_path = py_path,
            .cache_dir = cache_dir,
        };
    }

    // Cache miss - extract files
    std.fs.makeDirAbsolute(cache_dir) catch |err| switch (err) {
        error.PathAlreadyExists => {},
        else => return err,
    };

    // Extract MicroPython binary
    {
        const mpy_file = try std.fs.createFileAbsolute(mpy_path, .{});
        defer mpy_file.close();

        try self_file.seekTo(trailer.micropython_offset);
        var remaining = trailer.micropython_size;
        var buf: [65536]u8 = undefined;
        while (remaining > 0) {
            const to_read = @min(remaining, buf.len);
            const bytes_read = try self_file.read(buf[0..to_read]);
            if (bytes_read == 0) break;
            try mpy_file.writeAll(buf[0..bytes_read]);
            remaining -= bytes_read;
        }

        // Make executable
        try std.posix.fchmod(mpy_file.handle, 0o755);
    }

    // Extract Python code
    {
        const py_file = try std.fs.createFileAbsolute(py_path, .{});
        defer py_file.close();

        try self_file.seekTo(trailer.python_offset);
        var remaining = trailer.python_size;
        var buf: [65536]u8 = undefined;
        while (remaining > 0) {
            const to_read = @min(remaining, buf.len);
            const bytes_read = try self_file.read(buf[0..to_read]);
            if (bytes_read == 0) break;
            try py_file.writeAll(buf[0..bytes_read]);
            remaining -= bytes_read;
        }
    }

    return ExecContext{
        .mpy_path = mpy_path,
        .py_path = py_path,
        .cache_dir = cache_dir,
    };
}

/// Fallback for systems without memfd_create
fn prepareFallback(
    allocator: std.mem.Allocator,
    self_file: std.fs.File,
    trailer: Trailer,
) !ExecContext {
    // Use simple /tmp extraction without caching
    const pid = std.os.linux.getpid();
    const cache_dir = try std.fmt.allocPrint(allocator, "/tmp/ucharm-{d}", .{pid});
    const mpy_path = try std.fmt.allocPrint(allocator, "{s}/m", .{cache_dir});
    const py_path = try std.fmt.allocPrint(allocator, "{s}/a.py", .{cache_dir});

    std.fs.makeDirAbsolute(cache_dir) catch |err| switch (err) {
        error.PathAlreadyExists => {},
        else => return err,
    };

    // Extract MicroPython binary
    {
        const mpy_file = try std.fs.createFileAbsolute(mpy_path, .{});
        defer mpy_file.close();

        try self_file.seekTo(trailer.micropython_offset);
        var remaining = trailer.micropython_size;
        var buf: [65536]u8 = undefined;
        while (remaining > 0) {
            const to_read = @min(remaining, buf.len);
            const bytes_read = try self_file.read(buf[0..to_read]);
            if (bytes_read == 0) break;
            try mpy_file.writeAll(buf[0..bytes_read]);
            remaining -= bytes_read;
        }

        try std.posix.fchmod(mpy_file.handle, 0o755);
    }

    // Extract Python code
    {
        const py_file = try std.fs.createFileAbsolute(py_path, .{});
        defer py_file.close();

        try self_file.seekTo(trailer.python_offset);
        var remaining = trailer.python_size;
        var buf: [65536]u8 = undefined;
        while (remaining > 0) {
            const to_read = @min(remaining, buf.len);
            const bytes_read = try self_file.read(buf[0..to_read]);
            if (bytes_read == 0) break;
            try py_file.writeAll(buf[0..bytes_read]);
            remaining -= bytes_read;
        }
    }

    return ExecContext{
        .mpy_path = mpy_path,
        .py_path = py_path,
        .cache_dir = null, // Don't cache pid-based paths
    };
}

/// Execute MicroPython with the Python script using std.process.Child
/// This spawns and waits, propagating exit code
pub fn exec(allocator: std.mem.Allocator, ctx: ExecContext, args: []const [*:0]const u8) noreturn {
    // Build argv as slices
    var argv_list: std.ArrayList([]const u8) = .empty;
    defer argv_list.deinit(allocator);

    argv_list.append(allocator, ctx.mpy_path) catch {
        std.debug.print("ucharm: out of memory\n", .{});
        std.process.exit(1);
    };

    argv_list.append(allocator, ctx.py_path) catch {
        std.debug.print("ucharm: out of memory\n", .{});
        std.process.exit(1);
    };

    // Add user arguments
    for (args) |arg| {
        // Convert null-terminated to slice
        const len = std.mem.len(arg);
        argv_list.append(allocator, arg[0..len]) catch {
            std.debug.print("ucharm: out of memory\n", .{});
            std.process.exit(1);
        };
    }

    // Spawn child process with inherited stdio
    var child = std.process.Child.init(argv_list.items, allocator);
    // Inherit stdin, stdout, stderr from parent
    child.stdin_behavior = .Inherit;
    child.stdout_behavior = .Inherit;
    child.stderr_behavior = .Inherit;

    child.spawn() catch {
        std.debug.print("ucharm: failed to spawn micropython\n", .{});
        std.process.exit(1);
    };

    const result = child.wait() catch {
        std.debug.print("ucharm: failed to wait for micropython\n", .{});
        std.process.exit(1);
    };

    switch (result) {
        .Exited => |code| std.process.exit(code),
        .Signal => |sig| std.process.exit(128 + @as(u8, @intCast(sig))),
        else => std.process.exit(1),
    }
}

/// Calculate content hash from MicroPython and Python data
pub fn calculateHash(self_file: std.fs.File, trailer: Trailer) ![8]u8 {
    var hasher = std.crypto.hash.Md5.init(.{});

    // Hash first 1KB of MicroPython
    try self_file.seekTo(trailer.micropython_offset);
    var buf: [1024]u8 = undefined;
    const mpy_to_hash = @min(trailer.micropython_size, 1024);
    const mpy_read = try self_file.read(buf[0..mpy_to_hash]);
    hasher.update(buf[0..mpy_read]);

    // Hash first 1KB of Python
    try self_file.seekTo(trailer.python_offset);
    const py_to_hash = @min(trailer.python_size, 1024);
    const py_read = try self_file.read(buf[0..py_to_hash]);
    hasher.update(buf[0..py_read]);

    var hash: [16]u8 = undefined;
    hasher.final(&hash);

    // Return first 8 bytes
    return hash[0..8].*;
}

test "content hash matches shared Rust migration fixture" {
    const runtime = @embedFile("fixtures/cache_runtime_v1.txt");
    const python = @embedFile("fixtures/cache_python_v1.py");
    const expected = try decodeHashFixture(@embedFile("fixtures/cache_hash_v1.hex"));

    var temporary = std.testing.tmpDir(.{});
    defer temporary.cleanup();

    var file = try temporary.dir.createFile("bundle", .{ .read = true });
    defer file.close();
    try file.writeAll(runtime);
    try file.writeAll(python);

    const trailer = Trailer{
        .micropython_offset = 0,
        .micropython_size = runtime.len,
        .python_offset = runtime.len,
        .python_size = python.len,
    };
    const actual = try calculateHash(file, trailer);
    try std.testing.expectEqualSlices(u8, &expected, &actual);
}

fn decodeHashFixture(text: []const u8) ![8]u8 {
    var bytes: [8]u8 = undefined;
    var digit_index: usize = 0;

    for (text) |char| {
        if (std.ascii.isWhitespace(char)) continue;
        if (digit_index >= bytes.len * 2) return error.InvalidHexLength;

        const nibble: u8 = switch (char) {
            '0'...'9' => char - '0',
            'a'...'f' => char - 'a' + 10,
            'A'...'F' => char - 'A' + 10,
            else => return error.InvalidHex,
        };
        const byte_index = digit_index / 2;
        if (digit_index % 2 == 0) {
            bytes[byte_index] = nibble << 4;
        } else {
            bytes[byte_index] |= nibble;
        }
        digit_index += 1;
    }

    if (digit_index != bytes.len * 2) return error.InvalidHexLength;
    return bytes;
}
