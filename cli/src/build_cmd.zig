const std = @import("std");
const fs = std.fs;
const Allocator = std.mem.Allocator;
const io = @import("io.zig");
const style = io.style;
const builtin = @import("builtin");

// Version is read from VERSION file at compile time
const version_raw = @embedFile("VERSION");
const VERSION = std.mem.trim(u8, version_raw, " \t\n\r");

// Embedded loader stubs for instant-startup universal binaries
const stub_macos_aarch64 = @embedFile("stubs/loader-macos-aarch64");
const stub_macos_x86_64 = @embedFile("stubs/loader-macos-x86_64");
const stub_linux_x86_64 = @embedFile("stubs/loader-linux-x86_64");
const stub_linux_aarch64 = @embedFile("stubs/loader-linux-aarch64");

// Embedded pocketpy-ucharm binaries (contains all native modules)
// Note: Only the host platform binary is embedded by default. For other targets,
// ucharm will use a cached runtime in ~/.ucharm/runtimes/, download it from a
// release, or (when running from a source checkout) build it locally via `zig`.
const pocketpy_macos_aarch64 = @embedFile("stubs/pocketpy-ucharm-macos-aarch64");

// Trailer format constants (must match loader/src/trailer.zig)
const TRAILER_MAGIC: *const [8]u8 = "MCHARM01";
const TRAILER_SIZE: usize = 48;

// Import style symbols for convenience
const check = style.symbols.check;
const arrow = style.symbols.arrow_right;
const bullet = style.symbols.bullet;

const Mode = enum {
    single,
    executable,
    universal,
};

const Target = enum {
    macos_aarch64,
    macos_x86_64,
    linux_x86_64,
    linux_aarch64,

    pub fn name(self: Target) []const u8 {
        return switch (self) {
            .macos_aarch64 => "macos-aarch64",
            .macos_x86_64 => "macos-x86_64",
            .linux_x86_64 => "linux-x86_64",
            .linux_aarch64 => "linux-aarch64",
        };
    }

    pub fn displayName(self: Target) []const u8 {
        return switch (self) {
            .macos_aarch64 => "macOS (Apple Silicon)",
            .macos_x86_64 => "macOS (Intel)",
            .linux_x86_64 => "Linux (x86_64)",
            .linux_aarch64 => "Linux (ARM64)",
        };
    }

    pub fn fromString(s: []const u8) ?Target {
        if (std.mem.eql(u8, s, "macos-aarch64") or std.mem.eql(u8, s, "macos-arm64")) {
            return .macos_aarch64;
        } else if (std.mem.eql(u8, s, "macos-x86_64") or std.mem.eql(u8, s, "macos-amd64")) {
            return .macos_x86_64;
        } else if (std.mem.eql(u8, s, "linux-x86_64") or std.mem.eql(u8, s, "linux-amd64")) {
            return .linux_x86_64;
        } else if (std.mem.eql(u8, s, "linux-aarch64") or std.mem.eql(u8, s, "linux-arm64")) {
            return .linux_aarch64;
        }
        return null;
    }

    pub fn fromHost() Target {
        const os = builtin.os.tag;
        const arch = builtin.cpu.arch;

        if (os == .macos) {
            if (arch == .aarch64) {
                return .macos_aarch64;
            } else {
                return .macos_x86_64;
            }
        } else {
            // Linux (and other Unix-like)
            if (arch == .aarch64) {
                return .linux_aarch64;
            } else {
                return .linux_x86_64;
            }
        }
    }

    pub fn loaderStub(self: Target) []const u8 {
        return switch (self) {
            .macos_aarch64 => stub_macos_aarch64,
            .macos_x86_64 => stub_macos_x86_64,
            .linux_x86_64 => stub_linux_x86_64,
            .linux_aarch64 => stub_linux_aarch64,
        };
    }

    pub fn runtimeFilename(self: Target) []const u8 {
        return switch (self) {
            .macos_aarch64 => "pocketpy-ucharm-macos-aarch64",
            .macos_x86_64 => "pocketpy-ucharm-macos-x86_64",
            .linux_x86_64 => "pocketpy-ucharm-linux-x86_64",
            .linux_aarch64 => "pocketpy-ucharm-linux-aarch64",
        };
    }
};

pub fn run(allocator: Allocator, args: []const [:0]const u8) !void {
    var script_path: ?[]const u8 = null;
    var output_path: ?[]const u8 = null;
    var mode: Mode = .universal;
    var target: ?Target = null;

    // Parse arguments
    var i: usize = 0;
    while (i < args.len) : (i += 1) {
        const arg = args[i];

        if (std.mem.eql(u8, arg, "-o") or std.mem.eql(u8, arg, "--output")) {
            i += 1;
            if (i >= args.len) {
                io.eprint(style.err_prefix ++ " -o requires an argument\n", .{});
                std.process.exit(1);
            }
            output_path = args[i];
        } else if (std.mem.eql(u8, arg, "-m") or std.mem.eql(u8, arg, "--mode")) {
            i += 1;
            if (i >= args.len) {
                io.eprint(style.err_prefix ++ " --mode requires an argument\n", .{});
                std.process.exit(1);
            }
            const mode_str = args[i];
            if (std.mem.eql(u8, mode_str, "single")) {
                mode = .single;
            } else if (std.mem.eql(u8, mode_str, "executable")) {
                mode = .executable;
            } else if (std.mem.eql(u8, mode_str, "universal")) {
                mode = .universal;
            } else {
                io.eprint(style.err_prefix ++ " Unknown mode '{s}'. Use: single, executable, universal\n", .{mode_str});
                std.process.exit(1);
            }
        } else if (std.mem.eql(u8, arg, "-t") or std.mem.eql(u8, arg, "--target")) {
            i += 1;
            if (i >= args.len) {
                io.eprint(style.err_prefix ++ " --target requires an argument\n", .{});
                std.process.exit(1);
            }
            const target_str = args[i];
            target = Target.fromString(target_str);
            if (target == null) {
                io.eprint(style.err_prefix ++ " Unknown target '{s}'\n", .{target_str});
                io.eprint("\nAvailable targets:\n", .{});
                io.eprint("  macos-aarch64  " ++ style.dim ++ "(macOS Apple Silicon)" ++ style.reset ++ "\n", .{});
                io.eprint("  macos-x86_64   " ++ style.dim ++ "(macOS Intel)" ++ style.reset ++ "\n", .{});
                io.eprint("  linux-x86_64   " ++ style.dim ++ "(Linux x86_64)" ++ style.reset ++ "\n", .{});
                io.eprint("  linux-aarch64  " ++ style.dim ++ "(Linux ARM64)" ++ style.reset ++ "\n", .{});
                std.process.exit(1);
            }
        } else if (std.mem.eql(u8, arg, "--targets")) {
            io.print("Available targets:\n", .{});
            io.print("  macos-aarch64  " ++ style.dim ++ "(macOS Apple Silicon)" ++ style.reset ++ "\n", .{});
            io.print("  macos-x86_64   " ++ style.dim ++ "(macOS Intel)" ++ style.reset ++ "\n", .{});
            io.print("  linux-x86_64   " ++ style.dim ++ "(Linux x86_64)" ++ style.reset ++ "\n", .{});
            io.print("  linux-aarch64  " ++ style.dim ++ "(Linux ARM64)" ++ style.reset ++ "\n", .{});
            return;
        } else if (std.mem.eql(u8, arg, "-h") or std.mem.eql(u8, arg, "--help")) {
            printHelp();
            return;
        } else if (std.mem.startsWith(u8, arg, "-")) {
            io.eprint(style.err_prefix ++ " Unknown option '{s}'\n", .{arg});
            std.process.exit(1);
        } else {
            script_path = arg;
        }
    }

    if (script_path == null) {
        io.eprint(style.err_prefix ++ " No input script specified\n", .{});
        io.eprint("Usage: ucharm build <script.py> -o <output> [--mode <mode>] [--target <target>]\n", .{});
        std.process.exit(1);
    }

    if (output_path == null) {
        io.eprint(style.err_prefix ++ " No output path specified (-o)\n", .{});
        std.process.exit(1);
    }

    // Default to host target
    const build_target = target orelse Target.fromHost();

    // Check if script exists
    const script = script_path.?;
    fs.cwd().access(script, .{}) catch {
        io.eprint(style.err_prefix ++ " Script not found: {s}\n", .{script});
        std.process.exit(1);
    };

    // Print header
    io.print("\n", .{});
    io.print(style.brand ++ style.bold ++ "μcharm build" ++ style.reset ++ "\n", .{});
    io.print(style.header_line ++ "\n", .{});
    io.print(style.dim ++ "  Input:  " ++ style.reset ++ "{s}\n", .{script});
    io.print(style.dim ++ "  Output: " ++ style.reset ++ "{s}\n", .{output_path.?});
    io.print(style.dim ++ "  Mode:   " ++ style.reset ++ style.brand ++ "{s}" ++ style.reset ++ "\n", .{@tagName(mode)});
    if (mode == .universal) {
        io.print(style.dim ++ "  Target: " ++ style.reset ++ style.brand ++ "{s}" ++ style.reset ++ style.dim ++ " ({s})" ++ style.reset ++ "\n", .{ build_target.name(), build_target.displayName() });
    }
    io.print(style.header_line ++ "\n\n", .{});

    switch (mode) {
        .single => try buildSingle(allocator, script, output_path.?),
        .executable => try buildExecutable(allocator, script, output_path.?),
        .universal => try buildUniversal(allocator, script, output_path.?, build_target),
    }
}

fn printHelp() void {
    io.print(
        \\{s}μcharm build{s} - Build standalone binaries from Python scripts
        \\
        \\{s}USAGE:{s}
        \\    ucharm build <script.py> -o <output> [OPTIONS]
        \\
        \\{s}OPTIONS:{s}
        \\    -o, --output <path>    Output file path (required)
        \\    -m, --mode <mode>      Build mode: universal, executable, single
        \\                           (default: universal)
        \\    -t, --target <target>  Target platform for cross-compilation
        \\                           (default: current platform)
        \\    --targets              List available targets
        \\    -h, --help             Show this help
        \\
        \\{s}TARGETS:{s}
        \\    macos-aarch64          macOS on Apple Silicon
        \\    macos-x86_64           macOS on Intel
        \\    linux-x86_64           Linux on x86_64
        \\    linux-aarch64          Linux on ARM64
        \\
        \\{s}MODES:{s}
        \\    universal              Standalone binary (~900KB, no dependencies)
        \\    executable             Shell wrapper (requires pocketpy-ucharm)
        \\    single                 Transformed .py file (requires pocketpy-ucharm)
        \\
        \\{s}EXAMPLES:{s}
        \\    ucharm build app.py -o app
        \\    ucharm build app.py -o app-linux --target linux-x86_64
        \\    ucharm build app.py -o app.py --mode single
        \\
    , .{ style.bold, style.reset, style.dim, style.reset, style.dim, style.reset, style.dim, style.reset, style.dim, style.reset, style.dim, style.reset });
}

fn transformScript(allocator: Allocator, script_path: []const u8) ![]u8 {
    // Read the user's script
    const script_content = try fs.cwd().readFileAlloc(allocator, script_path, 1024 * 1024);
    defer allocator.free(script_content);

    // Build output
    var output_buffer: std.ArrayList(u8) = .empty;
    errdefer output_buffer.deinit(allocator);

    // Header
    try output_buffer.appendSlice(allocator, "#!/usr/bin/env pocketpy-ucharm\n");
    try output_buffer.appendSlice(allocator, "# Built with ucharm - native modules edition\n\n");

    // Track what needs to be imported from native modules
    var needs_tui = false;
    var needs_input = false;

    // First pass: check what ucharm imports are used
    var check_lines = std.mem.splitSequence(u8, script_content, "\n");
    while (check_lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t");

        if (std.mem.startsWith(u8, trimmed, "from ucharm import")) {
            // Parse the imports to see what's needed
            const import_part = trimmed["from ucharm import".len..];
            if (containsAny(import_part, &.{ "style", "box", "rule", "success", "error", "warning", "info", "progress" })) {
                needs_tui = true;
            }
            if (containsAny(import_part, &.{ "select", "multiselect", "confirm", "prompt", "password" })) {
                needs_input = true;
            }
        } else if (std.mem.startsWith(u8, trimmed, "from ucharm.input import")) {
            // from ucharm.input import select, etc.
            needs_input = true;
        } else if (std.mem.startsWith(u8, trimmed, "from ucharm.components import") or
            std.mem.startsWith(u8, trimmed, "from ucharm.style import") or
            std.mem.startsWith(u8, trimmed, "from ucharm.table import"))
        {
            // from ucharm.components/style/table import ...
            needs_tui = true;
        } else if (std.mem.startsWith(u8, trimmed, "import ucharm")) {
            needs_tui = true;
            needs_input = true;
        }
    }

    // Add native module imports if needed
    if (needs_tui) {
        try output_buffer.appendSlice(allocator, "from tui import style, box, rule, success, error, warning, info, progress\n");
    }
    if (needs_input) {
        try output_buffer.appendSlice(allocator, "from input import select, multiselect, confirm, prompt, password\n");
    }
    if (needs_tui or needs_input) {
        try output_buffer.appendSlice(allocator, "\n");
    }

    // Second pass: process lines and skip ucharm imports
    var lines = std.mem.splitSequence(u8, script_content, "\n");
    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t");

        // Skip ucharm imports (we've replaced them with native module imports)
        if (std.mem.startsWith(u8, trimmed, "from ucharm import") or
            std.mem.startsWith(u8, trimmed, "import ucharm") or
            std.mem.startsWith(u8, trimmed, "from ucharm."))
        {
            continue;
        }

        // Skip sys.path modifications for ucharm
        if (std.mem.indexOf(u8, line, "sys.path") != null and
            std.mem.indexOf(u8, line, "ucharm") != null)
        {
            continue;
        }

        try output_buffer.appendSlice(allocator, line);
        try output_buffer.append(allocator, '\n');
    }

    return try output_buffer.toOwnedSlice(allocator);
}

fn containsAny(haystack: []const u8, needles: []const []const u8) bool {
    for (needles) |needle| {
        if (std.mem.indexOf(u8, haystack, needle) != null) {
            return true;
        }
    }
    return false;
}

fn buildSingle(allocator: Allocator, script: []const u8, output: []const u8) !void {
    // Transform script to use native modules
    const transformed = try transformScript(allocator, script);
    defer allocator.free(transformed);

    // Write output file
    const output_file = try fs.cwd().createFile(output, .{});
    defer output_file.close();
    try output_file.writeAll(transformed);

    // Make executable (mode 0o755)
    const file_for_chmod = try fs.cwd().openFile(output, .{ .mode = .read_write });
    defer file_for_chmod.close();
    try file_for_chmod.chmod(0o755);

    io.print(style.success ++ check ++ style.reset ++ " Transformed Python code " ++ style.dim ++ "({d} bytes)" ++ style.reset ++ "\n", .{transformed.len});
    io.print("\n" ++ style.dim ++ "Note: Requires pocketpy-ucharm with native modules" ++ style.reset ++ "\n", .{});
}

fn buildExecutable(allocator: Allocator, script: []const u8, output: []const u8) !void {
    // Transform script
    const transformed = try transformScript(allocator, script);
    defer allocator.free(transformed);

    // Base64 encode
    const encoder = std.base64.standard.Encoder;
    const encoded_len = encoder.calcSize(transformed.len);
    const encoded = try allocator.alloc(u8, encoded_len);
    defer allocator.free(encoded);
    _ = encoder.encode(encoded, transformed);

    // Create shell wrapper that extracts embedded pocketpy
    var wrapper: std.ArrayList(u8) = .empty;
    defer wrapper.deinit(allocator);

    try wrapper.appendSlice(allocator, "#!/bin/bash\n");
    try wrapper.appendSlice(allocator, "# Built with μcharm - https://github.com/ucharmdev/ucharm\n");
    try wrapper.appendSlice(allocator, "# Requires pocketpy-ucharm with native modules\n\n");
    try wrapper.appendSlice(allocator, "POCKETPY=\"pocketpy-ucharm\"\n");
    try wrapper.appendSlice(allocator, "if ! command -v \"$POCKETPY\" &> /dev/null; then\n");
    try wrapper.appendSlice(allocator, "    POCKETPY=\"pocketpy\"\n");
    try wrapper.appendSlice(allocator, "    if ! command -v \"$POCKETPY\" &> /dev/null; then\n");
    try wrapper.appendSlice(allocator, "        echo \"Error: pocketpy not found\" >&2\n");
    try wrapper.appendSlice(allocator, "        exit 1\n");
    try wrapper.appendSlice(allocator, "    fi\n");
    try wrapper.appendSlice(allocator, "fi\n");
    try wrapper.appendSlice(allocator, "echo \"");
    try wrapper.appendSlice(allocator, encoded);
    try wrapper.appendSlice(allocator, "\" | base64 -d | \"$POCKETPY\" /dev/stdin \"$@\"\n");

    // Write wrapper
    const output_file = try fs.cwd().createFile(output, .{});
    defer output_file.close();
    try output_file.writeAll(wrapper.items);

    const file_for_chmod = try fs.cwd().openFile(output, .{ .mode = .read_write });
    defer file_for_chmod.close();
    try file_for_chmod.chmod(0o755);

    io.print(style.success ++ check ++ style.reset ++ " Created shell wrapper " ++ style.dim ++ "({d} bytes)" ++ style.reset ++ "\n", .{wrapper.items.len});
    io.print("\n" ++ style.header_line ++ "\n", .{});
    io.print(style.success ++ style.bold ++ check ++ " Built successfully!" ++ style.reset ++ "\n", .{});
    if (output[0] == '/') {
        io.print(style.dim ++ "  Run with: " ++ style.reset ++ "{s}\n\n", .{output});
    } else {
        io.print(style.dim ++ "  Run with: " ++ style.reset ++ "./{s}\n\n", .{output});
    }
}

fn buildUniversal(allocator: Allocator, script: []const u8, output: []const u8, target: Target) !void {
    // Transform script to use native modules
    const py_content = try transformScript(allocator, script);
    defer allocator.free(py_content);

    // Get pocketpy binary for target
    const runtime_binary = try getRuntimeBinary(allocator, target);
    const runtime_is_allocated = runtime_binary.allocated;
    defer if (runtime_is_allocated) allocator.free(runtime_binary.data);

    io.print(style.success ++ check ++ style.reset ++ " Using " ++ style.bold ++ "pocketpy-ucharm" ++ style.reset ++ style.dim ++ " for {s} ({d} KB)" ++ style.reset ++ "\n", .{ target.name(), runtime_binary.data.len / 1024 });

    // Select loader stub for target platform
    const stub = target.loaderStub();
    io.print(style.success ++ check ++ style.reset ++ " Selected loader " ++ style.bold ++ "{s}" ++ style.reset ++ style.dim ++ " ({d} KB)" ++ style.reset ++ "\n", .{ target.name(), stub.len / 1024 });

    // Calculate offsets for trailer
    const stub_size: u64 = stub.len;
    const runtime_offset: u64 = stub_size;
    const runtime_size: u64 = runtime_binary.data.len;
    const python_offset: u64 = runtime_offset + runtime_size;
    const python_size: u64 = py_content.len;

    // Build trailer (48 bytes)
    var trailer: [TRAILER_SIZE]u8 = undefined;
    @memcpy(trailer[0..8], TRAILER_MAGIC);
    std.mem.writeInt(u64, trailer[8..16], runtime_offset, .little);
    std.mem.writeInt(u64, trailer[16..24], runtime_size, .little);
    std.mem.writeInt(u64, trailer[24..32], python_offset, .little);
    std.mem.writeInt(u64, trailer[32..40], python_size, .little);
    @memcpy(trailer[40..48], TRAILER_MAGIC);

    // Write universal binary: [stub][runtime][python][trailer]
    const output_file = try fs.cwd().createFile(output, .{});
    defer output_file.close();

    try output_file.writeAll(stub);
    try output_file.writeAll(runtime_binary.data);
    try output_file.writeAll(py_content);
    try output_file.writeAll(&trailer);

    const file_for_chmod = try fs.cwd().openFile(output, .{ .mode = .read_write });
    defer file_for_chmod.close();
    try file_for_chmod.chmod(0o755);

    const total_size = stub.len + runtime_binary.data.len + py_content.len + TRAILER_SIZE;
    const total_kb = total_size / 1024;
    io.print(style.success ++ check ++ style.reset ++ " Wrote universal binary " ++ style.dim ++ "({d} KB)" ++ style.reset ++ "\n", .{total_kb});

    // Success summary
    io.print("\n" ++ style.header_line ++ "\n", .{});
    io.print(style.success ++ style.bold ++ check ++ " Built successfully!" ++ style.reset ++ "\n", .{});
    io.print(style.dim ++ "  Output:  " ++ style.reset ++ "{s}\n", .{output});
    io.print(style.dim ++ "  Target:  " ++ style.reset ++ "{s}\n", .{target.displayName()});
    io.print(style.dim ++ "  Size:    " ++ style.reset ++ "{d} KB " ++ style.dim ++ "(standalone, no dependencies)" ++ style.reset ++ "\n", .{total_kb});
    io.print(style.dim ++ "  Startup: " ++ style.reset ++ "~6ms " ++ style.dim ++ "(instant)" ++ style.reset ++ "\n", .{});
    // Show run command - handle absolute vs relative paths
    if (output[0] == '/') {
        io.print("\n" ++ style.dim ++ "  Run with: " ++ style.reset ++ "{s}\n\n", .{output});
    } else {
        io.print("\n" ++ style.dim ++ "  Run with: " ++ style.reset ++ "./{s}\n\n", .{output});
    }
}

const RuntimeBinary = struct {
    data: []const u8,
    allocated: bool,
};

fn getRuntimeBinary(allocator: Allocator, target: Target) !RuntimeBinary {
    // For macOS ARM64, we have the binary embedded
    if (target == .macos_aarch64) {
        return .{ .data = pocketpy_macos_aarch64, .allocated = false };
    }

    // For other targets, try to load from ~/.ucharm/runtimes/
    const home = std.posix.getenv("HOME") orelse "/tmp";
    const runtime_dir = try std.fmt.allocPrint(allocator, "{s}/.ucharm/runtimes", .{home});
    defer allocator.free(runtime_dir);
    const runtime_path = try std.fmt.allocPrint(allocator, "{s}/{s}", .{ runtime_dir, target.runtimeFilename() });
    defer allocator.free(runtime_path);
    const version_path = try std.fmt.allocPrint(allocator, "{s}.version", .{runtime_path});
    defer allocator.free(version_path);

    // Development-friendly fallback: if we're in a source checkout with PocketPy,
    // build the runtime for the target on-demand instead of downloading.
    if (runtimeExists(runtime_path) == false and pocketpySourceAvailable()) {
        return buildRuntimeLocally(allocator, target, runtime_dir, runtime_path, version_path);
    }

    // Try to open existing runtime
    const file = fs.cwd().openFile(runtime_path, .{}) catch |err| {
        if (err == error.FileNotFound) {
            // Runtime not found - offer to download
            return downloadRuntime(allocator, target, runtime_dir, runtime_path, version_path, false);
        }
        return err;
    };
    defer file.close();

    // Check version
    const cached_version = readVersionFile(allocator, version_path);
    defer if (cached_version) |v| allocator.free(v);
    if (cached_version) |ver| {
        if (!std.mem.eql(u8, ver, VERSION)) {
            // Version mismatch - offer to update
            io.print(style.warning ++ "!" ++ style.reset ++ " Cached runtime for " ++ style.bold ++ "{s}" ++ style.reset ++ " is version " ++ style.dim ++ "{s}" ++ style.reset ++ ", CLI is " ++ style.dim ++ "{s}" ++ style.reset ++ "\n", .{ target.name(), ver, VERSION });

            if (pocketpySourceAvailable()) {
                return buildRuntimeLocally(allocator, target, runtime_dir, runtime_path, version_path);
            }

            return downloadRuntime(allocator, target, runtime_dir, runtime_path, version_path, true);
        }
    } else {
        // No version file - offer to re-download to get proper versioning
        io.print(style.warning ++ "!" ++ style.reset ++ " Cached runtime for " ++ style.bold ++ "{s}" ++ style.reset ++ " has no version info\n", .{target.name()});
        if (pocketpySourceAvailable()) {
            return buildRuntimeLocally(allocator, target, runtime_dir, runtime_path, version_path);
        }
        return downloadRuntime(allocator, target, runtime_dir, runtime_path, version_path, true);
    }

    const stat = try file.stat();
    const data = try allocator.alloc(u8, stat.size);
    const bytes_read = try file.readAll(data);
    if (bytes_read != stat.size) {
        return error.IncompleteRead;
    }

    return .{ .data = data, .allocated = true };
}

fn readVersionFile(allocator: Allocator, path: []const u8) ?[]u8 {
    const file = fs.cwd().openFile(path, .{}) catch return null;
    defer file.close();

    var buf: [64]u8 = undefined;
    const n = file.readAll(&buf) catch return null;
    if (n == 0) return null;

    const content = std.mem.trim(u8, buf[0..n], " \t\n\r");
    if (content.len == 0) return null;

    const out = allocator.alloc(u8, content.len) catch return null;
    @memcpy(out, content);
    return out;
}

fn downloadRuntime(allocator: Allocator, target: Target, runtime_dir: []const u8, runtime_path: []const u8, version_path: []const u8, is_update: bool) !RuntimeBinary {
    // Use version-specific URL to ensure we get the matching runtime
    const download_url = try std.fmt.allocPrint(allocator, "https://github.com/ucharmdev/ucharm/releases/download/v{s}/{s}", .{ VERSION, target.runtimeFilename() });
    defer allocator.free(download_url);

    const checksum_url = try std.fmt.allocPrint(allocator, "{s}.sha256", .{download_url});
    defer allocator.free(checksum_url);

    if (is_update) {
        io.print("  Update to version " ++ style.bold ++ "{s}" ++ style.reset ++ "? " ++ style.dim ++ "(~850KB)" ++ style.reset ++ " [Y/n] ", .{VERSION});
    } else {
        io.print(style.warning ++ "?" ++ style.reset ++ " PocketPy runtime for " ++ style.bold ++ "{s}" ++ style.reset ++ " not found locally.\n", .{target.name()});
        io.print("  Download version " ++ style.bold ++ "{s}" ++ style.reset ++ " from GitHub? " ++ style.dim ++ "(~850KB)" ++ style.reset ++ " [Y/n] ", .{VERSION});
    }

    // Read user input
    var buf: [10]u8 = undefined;
    const read_result = std.posix.read(std.posix.STDIN_FILENO, &buf) catch 0;

    // Default to yes, or check for explicit no
    const should_download = if (read_result == 0) true else blk: {
        const input = std.mem.trim(u8, buf[0..read_result], " \t\n\r");
        break :blk input.len == 0 or input[0] == 'y' or input[0] == 'Y';
    };

    if (!should_download) {
        io.print("\n" ++ style.dim ++ "To download manually:" ++ style.reset ++ "\n", .{});
        io.print("  mkdir -p {s}\n", .{runtime_dir});
        io.print("  curl -L {s} -o {s}\n", .{ download_url, runtime_path });
        io.print("  echo '{s}' > {s}\n\n", .{ VERSION, version_path });
        std.process.exit(1);
    }

    io.print("\n", .{});

    // Create runtime directory
    fs.cwd().makePath(runtime_dir) catch |err| {
        io.eprint(style.err_prefix ++ " Failed to create directory {s}: {}\n", .{ runtime_dir, err });
        std.process.exit(1);
    };

    // Fetch expected sha256 for integrity verification.
    const expected_sha256 = downloadAndParseSha256(allocator, checksum_url) catch |err| {
        io.eprint(style.err_prefix ++ " Failed to fetch runtime checksum: {}\n", .{err});
        io.eprint(style.dim ++ "Expected checksum asset: {s}\n" ++ style.reset, .{checksum_url});
        std.process.exit(1);
    };
    defer allocator.free(expected_sha256);

    // Download using curl (available on macOS and most Linux)
    io.print(style.dim ++ "  Downloading..." ++ style.reset, .{});

    var child = std.process.Child.init(&[_][]const u8{
        "curl",
        "-fSL",
        "--progress-bar",
        "-o",
        runtime_path,
        download_url,
    }, allocator);
    child.stderr_behavior = .Inherit;
    child.stdout_behavior = .Inherit;

    _ = child.spawn() catch |err| {
        io.eprint("\n" ++ style.err_prefix ++ "Failed to run curl: {}\n", .{err});
        io.eprint(style.dim ++ "Install curl or download manually:\n" ++ style.reset, .{});
        io.eprint("  curl -L {s} -o {s}\n", .{ download_url, runtime_path });
        std.process.exit(1);
    };

    const result = child.wait() catch |err| {
        io.eprint("\n" ++ style.err_prefix ++ "Download failed: {}\n", .{err});
        std.process.exit(1);
    };

    if (result.Exited != 0) {
        if (pocketpySourceAvailable()) {
            io.eprint("\n" ++ style.warning ++ "!" ++ style.reset ++ " Download failed; building PocketPy runtime locally instead.\n", .{});
            return buildRuntimeLocally(allocator, target, runtime_dir, runtime_path, version_path);
        }
        io.eprint("\n" ++ style.err_prefix ++ "Download failed (curl exit code: {})\n", .{result.Exited});
        io.eprint(style.dim ++ "The runtime may not be available yet. Try again after the next release.\n" ++ style.reset, .{});
        std.process.exit(1);
    }

    io.print(" " ++ style.success ++ check ++ style.reset ++ "\n", .{});

    // Verify integrity before using the runtime.
    if (!verifyFileSha256(runtime_path, expected_sha256)) {
        fs.cwd().deleteFile(runtime_path) catch {};
        io.eprint(style.err_prefix ++ " Runtime checksum mismatch for {s}\n", .{target.runtimeFilename()});
        io.eprint(style.dim ++ "Expected sha256: {s}\n" ++ style.reset, .{expected_sha256});
        std.process.exit(1);
    }

    // Make it executable
    const file_for_chmod = try fs.cwd().openFile(runtime_path, .{ .mode = .read_write });
    defer file_for_chmod.close();
    try file_for_chmod.chmod(0o755);

    // Write version file
    const ver_file = try fs.cwd().createFile(version_path, .{});
    defer ver_file.close();
    try ver_file.writeAll(VERSION);
    try ver_file.writeAll("\n");

    // Now read and return the downloaded file
    const file = try fs.cwd().openFile(runtime_path, .{});
    defer file.close();

    const stat = try file.stat();
    const data = try allocator.alloc(u8, stat.size);
    const bytes_read = try file.readAll(data);
    if (bytes_read != stat.size) {
        return error.IncompleteRead;
    }

    io.print(style.success ++ check ++ style.reset ++ " Downloaded runtime " ++ style.dim ++ "v{s}" ++ style.reset ++ " to " ++ style.dim ++ "{s}" ++ style.reset ++ "\n\n", .{ VERSION, runtime_path });

    return .{ .data = data, .allocated = true };
}

fn downloadAndParseSha256(allocator: Allocator, url: []const u8) ![]u8 {
    var child = std.process.Child.init(&[_][]const u8{
        "curl",
        "-fsSL",
        url,
    }, allocator);
    child.stderr_behavior = .Inherit;
    child.stdout_behavior = .Pipe;

    try child.spawn();

    var out: std.ArrayList(u8) = .empty;
    errdefer out.deinit(allocator);

    if (child.stdout) |stdout_file| {
        defer stdout_file.close();

        var buf: [4096]u8 = undefined;
        while (true) {
            const n = try stdout_file.read(&buf);
            if (n == 0) break;
            if (out.items.len + n > 16 * 1024) return error.StreamTooLong;
            try out.appendSlice(allocator, buf[0..n]);
        }
    }

    const result = try child.wait();
    if (result.Exited != 0) return error.ChecksumDownloadFailed;

    const trimmed = std.mem.trim(u8, out.items, " \t\n\r");
    if (trimmed.len == 0) return error.InvalidChecksum;

    const end = std.mem.indexOfAny(u8, trimmed, " \t") orelse trimmed.len;
    const token = trimmed[0..end];
    if (token.len != 64) return error.InvalidChecksum;

    // Validate hex.
    for (token) |c| {
        const ok = (c >= '0' and c <= '9') or (c >= 'a' and c <= 'f') or (c >= 'A' and c <= 'F');
        if (!ok) return error.InvalidChecksum;
    }

    const out_hash = try allocator.alloc(u8, token.len);
    @memcpy(out_hash, token);
    return out_hash;
}

fn verifyFileSha256(path: []const u8, expected_hex: []const u8) bool {
    var file = fs.cwd().openFile(path, .{}) catch return false;
    defer file.close();

    var hasher = std.crypto.hash.sha2.Sha256.init(.{});
    var buf: [16 * 1024]u8 = undefined;
    while (true) {
        const n = file.read(&buf) catch return false;
        if (n == 0) break;
        hasher.update(buf[0..n]);
    }

    var digest: [32]u8 = undefined;
    hasher.final(&digest);

    const hex_buf = std.fmt.bytesToHex(digest, .lower);
    return std.ascii.eqlIgnoreCase(hex_buf[0..], expected_hex);
}

fn runtimeExists(path: []const u8) bool {
    fs.cwd().access(path, .{}) catch return false;
    return true;
}

fn pocketpySourceAvailable() bool {
    const override = std.posix.getenv("UCHARM_POCKETPY_DIR");
    if (override) |dir| {
        const build_zig = std.fmt.allocPrint(std.heap.page_allocator, "{s}/build.zig", .{dir}) catch return false;
        defer std.heap.page_allocator.free(build_zig);
        return runtimeExists(build_zig);
    }
    return runtimeExists("pocketpy/build.zig");
}

fn buildRuntimeLocally(allocator: Allocator, target: Target, runtime_dir: []const u8, runtime_path: []const u8, version_path: []const u8) !RuntimeBinary {
    const pocketpy_dir = std.posix.getenv("UCHARM_POCKETPY_DIR") orelse "pocketpy";

    io.print(style.dim ++ "  Building PocketPy runtime locally..." ++ style.reset ++ "\n", .{});

    fs.cwd().makePath(runtime_dir) catch |err| {
        io.eprint(style.err_prefix ++ " Failed to create directory {s}: {}\n", .{ runtime_dir, err });
        std.process.exit(1);
    };

    const zig_target = switch (target) {
        .macos_aarch64 => "aarch64-macos",
        .macos_x86_64 => "x86_64-macos",
        .linux_x86_64 => "x86_64-linux-musl",
        .linux_aarch64 => "aarch64-linux-musl",
    };

    var child = std.process.Child.init(&[_][]const u8{
        "zig",
        "build",
        "-Doptimize=ReleaseSmall",
        try std.fmt.allocPrint(allocator, "-Dtarget={s}", .{zig_target}),
    }, allocator);
    defer allocator.free(child.argv[3]);

    child.cwd = pocketpy_dir;
    child.stderr_behavior = .Inherit;
    child.stdout_behavior = .Inherit;

    _ = child.spawn() catch |err| {
        io.eprint(style.err_prefix ++ " Failed to run zig to build PocketPy runtime: {}\n", .{err});
        io.eprint(style.dim ++ "Expected PocketPy source at: {s}\n" ++ style.reset, .{pocketpy_dir});
        std.process.exit(1);
    };

    const result = child.wait() catch |err| {
        io.eprint(style.err_prefix ++ " Local runtime build failed: {}\n", .{err});
        std.process.exit(1);
    };
    if (result.Exited != 0) {
        io.eprint(style.err_prefix ++ " Local runtime build failed (exit code: {})\n", .{result.Exited});
        std.process.exit(1);
    }

    const built_path = try std.fmt.allocPrint(allocator, "{s}/zig-out/bin/pocketpy-ucharm", .{pocketpy_dir});
    defer allocator.free(built_path);

    fs.cwd().copyFile(built_path, fs.cwd(), runtime_path, .{}) catch |err| {
        io.eprint(style.err_prefix ++ " Failed to copy built runtime from {s} to {s}: {}\n", .{ built_path, runtime_path, err });
        std.process.exit(1);
    };

    // Make it executable
    const file_for_chmod = try fs.cwd().openFile(runtime_path, .{ .mode = .read_write });
    defer file_for_chmod.close();
    try file_for_chmod.chmod(0o755);

    // Write version file (ties cached runtime to the CLI version)
    const ver_file = try fs.cwd().createFile(version_path, .{});
    defer ver_file.close();
    try ver_file.writeAll(VERSION);
    try ver_file.writeAll("\n");

    // Read and return the runtime data
    const file = try fs.cwd().openFile(runtime_path, .{});
    defer file.close();

    const stat = try file.stat();
    const data = try allocator.alloc(u8, stat.size);
    const bytes_read = try file.readAll(data);
    if (bytes_read != stat.size) {
        return error.IncompleteRead;
    }

    io.print(style.success ++ check ++ style.reset ++ " Built runtime for " ++ style.dim ++ "{s}" ++ style.reset ++ " to " ++ style.dim ++ "{s}" ++ style.reset ++ "\n", .{ target.name(), runtime_path });

    return .{ .data = data, .allocated = true };
}
