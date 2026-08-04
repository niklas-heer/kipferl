const std = @import("std");
const fs = std.fs;
const io = @import("io.zig");

// Embedded type stubs - generated from runtime modules
const stub_ansi = @embedFile("stubs/ansi.pyi");
const stub_args = @embedFile("stubs/args.pyi");
const stub_base64 = @embedFile("stubs/base64.pyi");
const stub_tui = @embedFile("stubs/tui.pyi");
const stub_copy = @embedFile("stubs/copy.pyi");
const stub_csv = @embedFile("stubs/csv.pyi");
const stub_datetime = @embedFile("stubs/datetime.pyi");
const stub_fnmatch = @embedFile("stubs/fnmatch.pyi");
const stub_functools = @embedFile("stubs/functools.pyi");
const stub_glob = @embedFile("stubs/glob.pyi");
const stub_heapq = @embedFile("stubs/heapq.pyi");
const stub_input = @embedFile("stubs/input.pyi");
const stub_itertools = @embedFile("stubs/itertools.pyi");
const stub_logging = @embedFile("stubs/logging.pyi");
const stub_operator = @embedFile("stubs/operator.pyi");
const stub_random = @embedFile("stubs/random.pyi");
const stub_shutil = @embedFile("stubs/shutil.pyi");
const stub_signal = @embedFile("stubs/signal.pyi");
const stub_statistics = @embedFile("stubs/statistics.pyi");
const stub_subprocess = @embedFile("stubs/subprocess.pyi");
const stub_tempfile = @embedFile("stubs/tempfile.pyi");
const stub_term = @embedFile("stubs/term.pyi");
const stub_textwrap = @embedFile("stubs/textwrap.pyi");
const stub_typing = @embedFile("stubs/typing.pyi");

pub const stubs = [_]struct { name: []const u8, content: []const u8 }{
    .{ .name = "ansi.pyi", .content = stub_ansi },
    .{ .name = "args.pyi", .content = stub_args },
    .{ .name = "base64.pyi", .content = stub_base64 },
    .{ .name = "tui.pyi", .content = stub_tui },
    .{ .name = "copy.pyi", .content = stub_copy },
    .{ .name = "csv.pyi", .content = stub_csv },
    .{ .name = "datetime.pyi", .content = stub_datetime },
    .{ .name = "fnmatch.pyi", .content = stub_fnmatch },
    .{ .name = "functools.pyi", .content = stub_functools },
    .{ .name = "glob.pyi", .content = stub_glob },
    .{ .name = "heapq.pyi", .content = stub_heapq },
    .{ .name = "input.pyi", .content = stub_input },
    .{ .name = "itertools.pyi", .content = stub_itertools },
    .{ .name = "logging.pyi", .content = stub_logging },
    .{ .name = "operator.pyi", .content = stub_operator },
    .{ .name = "random.pyi", .content = stub_random },
    .{ .name = "shutil.pyi", .content = stub_shutil },
    .{ .name = "signal.pyi", .content = stub_signal },
    .{ .name = "statistics.pyi", .content = stub_statistics },
    .{ .name = "subprocess.pyi", .content = stub_subprocess },
    .{ .name = "tempfile.pyi", .content = stub_tempfile },
    .{ .name = "term.pyi", .content = stub_term },
    .{ .name = "textwrap.pyi", .content = stub_textwrap },
    .{ .name = "typing.pyi", .content = stub_typing },
};

// AI instruction templates - embedded from cli/src/templates/
pub const agents_md = @embedFile("templates/AGENTS.md");
pub const claude_md = @embedFile("templates/CLAUDE.md");
pub const copilot_instructions = @embedFile("templates/copilot-instructions.md");

pub const pyrightconfig =
    \\{
    \\  "include": ["."],
    \\  "exclude": [".ucharm"],
    \\  "stubPath": ".ucharm/stubs",
    \\  "reportMissingImports": false,
    \\  "reportMissingModuleSource": false,
    \\  "pythonVersion": "3.11",
    \\  "typeCheckingMode": "basic"
    \\}
    \\
;

// Project template for new projects
pub const app_template =
    \\#!/usr/bin/env python3
    \\"""
    \\{s} - Built with ucharm
    \\"""
    \\from ucharm import box, success, error, warning, info
    \\from ucharm import select, confirm, prompt
    \\
    \\
    \\def main():
    \\    box(
    \\        "{s}\n"
    \\        "Built with ucharm",
    \\        title="Welcome",
    \\        border_color="cyan"
    \\    )
    \\    print()
    \\
    \\    choice = select("What would you like to do?", [
    \\        "Say hello",
    \\        "Show status messages",
    \\        "Exit"
    \\    ])
    \\
    \\    if choice == "Say hello":
    \\        name = prompt("What's your name?", default="World")
    \\        print()
    \\        success(f"Hello, {{name}}!")
    \\    elif choice == "Show status messages":
    \\        print()
    \\        success("This is a success message")
    \\        warning("This is a warning message")
    \\        error("This is an error message")
    \\        info("This is an info message")
    \\    else:
    \\        info("Goodbye!")
    \\
    \\
    \\if __name__ == "__main__":
    \\    main()
    \\
;

pub const Options = struct {
    add_stubs: bool = false,
    ai_type: ?[]const u8 = null, // "agents", "claude", "copilot", "all"
    create_app: bool = false,
    app_name: ?[]const u8 = null,
};

/// Initialize a ucharm project in the given directory
/// If dir is null, uses current working directory
pub fn init(dir: ?fs.Dir, options: Options) !usize {
    const target_dir = dir orelse fs.cwd();
    var files_created: usize = 0;

    // Create app file if requested
    if (options.create_app) {
        if (options.app_name) |name| {
            try createAppFile(target_dir, name, &files_created);
        }
    }

    // Create stubs
    if (options.add_stubs) {
        try createStubs(target_dir, &files_created);
        try createPyrightConfig(target_dir, &files_created);
    }

    // Create AI instructions
    if (options.ai_type) |ai| {
        if (std.mem.eql(u8, ai, "agents") or std.mem.eql(u8, ai, "all")) {
            try writeFileIfNotExists(target_dir, "AGENTS.md", agents_md, &files_created);
        }
        if (std.mem.eql(u8, ai, "claude") or std.mem.eql(u8, ai, "all")) {
            try writeFileIfNotExists(target_dir, "CLAUDE.md", claude_md, &files_created);
        }
        if (std.mem.eql(u8, ai, "copilot") or std.mem.eql(u8, ai, "all")) {
            try createParentDirs(target_dir, ".github/copilot-instructions.md");
            try writeFileIfNotExists(target_dir, ".github/copilot-instructions.md", copilot_instructions, &files_created);
        }
    }

    return files_created;
}

fn createAppFile(dir: fs.Dir, name: []const u8, count: *usize) !void {
    // Convert name to filename (lowercase, underscores)
    var filename_buf: [256]u8 = undefined;
    var filename_len: usize = 0;

    for (name) |c| {
        if (c == ' ' or c == '-') {
            filename_buf[filename_len] = '_';
        } else if (c >= 'A' and c <= 'Z') {
            filename_buf[filename_len] = c + 32; // lowercase
        } else {
            filename_buf[filename_len] = c;
        }
        filename_len += 1;
        if (filename_len >= filename_buf.len - 4) break;
    }

    const filename_base = filename_buf[0..filename_len];

    // Create filename with .py extension
    var full_filename: [260]u8 = undefined;
    const filename = std.fmt.bufPrint(&full_filename, "{s}.py", .{filename_base}) catch {
        io.eprint("\x1b[31mError:\x1b[0m Filename too long\n", .{});
        return;
    };

    // Check if file exists
    if (dir.access(filename, .{})) |_| {
        io.print("\x1b[2m-\x1b[0m {s} already exists (skipped)\n", .{filename});
        return;
    } else |_| {}

    // Generate content
    var content_buf: [4096]u8 = undefined;
    const content = std.fmt.bufPrint(&content_buf, app_template, .{ name, name }) catch {
        io.eprint("\x1b[31mError:\x1b[0m Template error\n", .{});
        return;
    };

    // Write file
    const file = dir.createFile(filename, .{}) catch |err| {
        io.eprint("\x1b[33mWarning:\x1b[0m Could not create {s}: {}\n", .{ filename, err });
        return;
    };
    defer file.close();
    file.writeAll(content) catch {};

    // Make executable
    const file_for_chmod = dir.openFile(filename, .{ .mode = .read_write }) catch return;
    defer file_for_chmod.close();
    file_for_chmod.chmod(0o755) catch {};

    io.print("\x1b[32m+\x1b[0m Created {s}\n", .{filename});
    count.* += 1;
}

fn createStubs(dir: fs.Dir, count: *usize) !void {
    // Create .ucharm/stubs directory
    dir.makePath(".ucharm/stubs") catch |err| {
        io.eprint("\x1b[31mError:\x1b[0m Failed to create .ucharm/stubs: {}\n", .{err});
        return;
    };

    // Write stub files
    for (stubs) |stub| {
        var path_buf: [256]u8 = undefined;
        const path = std.fmt.bufPrint(&path_buf, ".ucharm/stubs/{s}", .{stub.name}) catch continue;

        const file = dir.createFile(path, .{}) catch |err| {
            io.eprint("\x1b[33mWarning:\x1b[0m Could not create {s}: {}\n", .{ path, err });
            continue;
        };
        defer file.close();
        file.writeAll(stub.content) catch {};
    }
    io.print("\x1b[32m+\x1b[0m Created .ucharm/stubs/ ({d} stub files)\n", .{stubs.len});
    count.* += 1;
}

fn createPyrightConfig(dir: fs.Dir, count: *usize) !void {
    const file = dir.createFile("pyrightconfig.json", .{ .exclusive = true }) catch |err| {
        if (err == error.PathAlreadyExists) {
            io.print("\x1b[2m-\x1b[0m pyrightconfig.json already exists (skipped)\n", .{});
            return;
        }
        io.eprint("\x1b[33mWarning:\x1b[0m Could not create pyrightconfig.json: {}\n", .{err});
        return;
    };
    defer file.close();
    file.writeAll(pyrightconfig) catch {};
    io.print("\x1b[32m+\x1b[0m Created pyrightconfig.json\n", .{});
    count.* += 1;
}

fn createParentDirs(dir: fs.Dir, path: []const u8) !void {
    if (std.mem.lastIndexOf(u8, path, "/")) |idx| {
        const parent = path[0..idx];
        dir.makePath(parent) catch {};
    }
}

fn writeFileIfNotExists(dir: fs.Dir, path: []const u8, content: []const u8, count: *usize) !void {
    const file = dir.createFile(path, .{ .exclusive = true }) catch |err| {
        if (err == error.PathAlreadyExists) {
            io.print("\x1b[2m-\x1b[0m {s} already exists (skipped)\n", .{path});
            return;
        }
        io.eprint("\x1b[33mWarning:\x1b[0m Could not create {s}: {}\n", .{ path, err });
        return;
    };
    defer file.close();
    file.writeAll(content) catch {};
    io.print("\x1b[32m+\x1b[0m Created {s}\n", .{path});
    count.* += 1;
}
