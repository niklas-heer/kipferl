const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const sqlite_amalgamation = b.dependency("sqlite_amalgamation", .{});

    const exe_mod = b.createModule(.{
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    const exe = b.addExecutable(.{
        .name = "pocketpy-ucharm",
        .root_module = exe_mod,
    });

    // Strip in release builds to keep the runtime small.
    if (optimize != .Debug) {
        exe.root_module.strip = true;
        exe.link_gc_sections = true;
    }

    const pk_mod = b.createModule(.{
        .root_source_file = b.path("src/pk.zig"),
        .target = target,
        .optimize = optimize,
    });
    pk_mod.addIncludePath(b.path("vendor"));

    // Core modules for ucharm (shared logic)
    const ansi_core = b.createModule(.{
        .root_source_file = b.path("../runtime/ucharm/ansi_core.zig"),
        .target = target,
        .optimize = optimize,
    });
    const args_core = b.createModule(.{
        .root_source_file = b.path("../runtime/ucharm/args_core.zig"),
        .target = target,
        .optimize = optimize,
    });
    const tui_core = b.createModule(.{
        .root_source_file = b.path("../runtime/ucharm/tui_core.zig"),
        .target = target,
        .optimize = optimize,
    });
    const input_core = b.createModule(.{
        .root_source_file = b.path("../runtime/ucharm/input_core.zig"),
        .target = target,
        .optimize = optimize,
    });

    exe.root_module.addImport("pk", pk_mod);
    exe.root_module.addImport("ansi_core", ansi_core);
    exe.root_module.addImport("args_core", args_core);
    exe.root_module.addImport("tui_core", tui_core);
    exe.root_module.addImport("input_core", input_core);

    // Compat modules
    const mod_fnmatch = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/fnmatch.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_fnmatch.addImport("pk", pk_mod);

    const mod_glob = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/glob.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_glob.addImport("pk", pk_mod);
    mod_glob.addImport("mod_fnmatch", mod_fnmatch);

    // Ucharm modules (with PocketPy bindings)
    const mod_ansi = b.createModule(.{
        .root_source_file = b.path("../runtime/ucharm/ansi.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_ansi.addImport("pk", pk_mod);
    mod_ansi.addImport("ansi_core", ansi_core);

    const mod_args = b.createModule(.{
        .root_source_file = b.path("../runtime/ucharm/args.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_args.addImport("pk", pk_mod);
    mod_args.addImport("args_core", args_core);

    const mod_tui = b.createModule(.{
        .root_source_file = b.path("../runtime/ucharm/tui.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_tui.addImport("pk", pk_mod);
    mod_tui.addImport("tui_core", tui_core);

    const mod_input = b.createModule(.{
        .root_source_file = b.path("../runtime/ucharm/input.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_input.addImport("pk", pk_mod);
    mod_input.addImport("input_core", input_core);

    const mod_term = b.createModule(.{
        .root_source_file = b.path("../runtime/ucharm/term.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_term.addImport("pk", pk_mod);

    const mod_template = b.createModule(.{
        .root_source_file = b.path("../runtime/ucharm/template.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_template.addImport("pk", pk_mod);
    mod_template.addIncludePath(b.path("vendor"));

    const mod_fetch = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/fetch.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_fetch.addImport("pk", pk_mod);
    mod_fetch.addIncludePath(b.path("vendor/bearssl/inc"));

    // Compat modules (CPython-compatible stdlib)
    const mod_sys = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/sys.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_sys.addImport("pk", pk_mod);

    const mod_argparse = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/argparse.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_argparse.addImport("pk", pk_mod);

    const mod_base64 = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/base64.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_base64.addImport("pk", pk_mod);

    const mod_csv = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/csv.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_csv.addImport("pk", pk_mod);

    const mod_datetime = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/datetime.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_datetime.addImport("pk", pk_mod);

    const mod_errno = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/errno.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_errno.addImport("pk", pk_mod);

    const mod_functools = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/functools.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_functools.addImport("pk", pk_mod);

    const mod_hashlib = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/hashlib.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_hashlib.addImport("pk", pk_mod);

    const mod_heapq = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/heapq.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_heapq.addImport("pk", pk_mod);

    const mod_io = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/io.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_io.addImport("pk", pk_mod);

    const mod_itertools = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/itertools.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_itertools.addImport("pk", pk_mod);

    const mod_logging = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/logging.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_logging.addImport("pk", pk_mod);

    const mod_math = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/math.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_math.addImport("pk", pk_mod);

    const mod_ospath = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/ospath.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_ospath.addImport("pk", pk_mod);

    const mod_os = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/os.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_os.addImport("pk", pk_mod);

    const mod_pathlib = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/pathlib.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_pathlib.addImport("pk", pk_mod);

    const mod_re = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/re.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_re.addImport("pk", pk_mod);

    const mod_shutil = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/shutil.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_shutil.addImport("pk", pk_mod);

    const mod_signal = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/signal.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_signal.addImport("pk", pk_mod);

    const mod_statistics = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/statistics.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_statistics.addImport("pk", pk_mod);

    const mod_str_ext = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/str_ext.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_str_ext.addImport("pk", pk_mod);

    const mod_strings = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/strings.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_strings.addImport("pk", pk_mod);

    const mod_struct = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/struct.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_struct.addImport("pk", pk_mod);

    const mod_subprocess = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/subprocess.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_subprocess.addImport("pk", pk_mod);

    const mod_tempfile = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/tempfile.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_tempfile.addImport("pk", pk_mod);

    const mod_textwrap = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/textwrap.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_textwrap.addImport("pk", pk_mod);

    const mod_typing = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/typing.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_typing.addImport("pk", pk_mod);

    const mod_operator = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/operator.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_operator.addImport("pk", pk_mod);

    const mod_random = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/random.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_random.addImport("pk", pk_mod);

    const mod_json = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/json.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_json.addImport("pk", pk_mod);

    const mod_time = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/time.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_time.addImport("pk", pk_mod);

    const mod_collections = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/collections.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_collections.addImport("pk", pk_mod);

    const mod_configparser = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/configparser.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_configparser.addImport("pk", pk_mod);

    const mod_unittest = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/unittest.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_unittest.addImport("pk", pk_mod);

    const mod_copy = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/copy.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_copy.addImport("pk", pk_mod);

    const mod_array = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/array.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_array.addImport("pk", pk_mod);

    const mod_binascii = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/binascii.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_binascii.addImport("pk", pk_mod);

    const mod_contextlib = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/contextlib.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_contextlib.addImport("pk", pk_mod);

    const mod_urllib_parse = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/urllib_parse.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_urllib_parse.addImport("pk", pk_mod);

    const mod_uuid = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/uuid.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_uuid.addImport("pk", pk_mod);

    const mod_toml = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/toml.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_toml.addImport("pk", pk_mod);

    const mod_secrets = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/secrets.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_secrets.addImport("pk", pk_mod);

    const mod_hmac = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/hmac.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_hmac.addImport("pk", pk_mod);

    const mod_dataclasses = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/dataclasses.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_dataclasses.addImport("pk", pk_mod);

    const mod_gzip = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/gzip.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_gzip.addImport("pk", pk_mod);

    const mod_zipfile = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/zipfile.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_zipfile.addImport("pk", pk_mod);

    const mod_tarfile = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/tarfile.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_tarfile.addImport("pk", pk_mod);

    const mod_xml_etree_elementtree = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/xml_etree_elementtree.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_xml_etree_elementtree.addImport("pk", pk_mod);

    const mod_sqlite3 = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/sqlite3.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_sqlite3.addImport("pk", pk_mod);
    mod_sqlite3.addIncludePath(sqlite_amalgamation.path("."));

    const mod_http_client = b.createModule(.{
        .root_source_file = b.path("../runtime/compat/http_client.zig"),
        .target = target,
        .optimize = optimize,
    });
    mod_http_client.addImport("pk", pk_mod);

    // Register all modules with executable
    exe.root_module.addImport("mod_fnmatch", mod_fnmatch);
    exe.root_module.addImport("mod_glob", mod_glob);
    exe.root_module.addImport("mod_ansi", mod_ansi);
    exe.root_module.addImport("mod_args", mod_args);
    exe.root_module.addImport("mod_tui", mod_tui);
    exe.root_module.addImport("mod_input", mod_input);
    exe.root_module.addImport("mod_term", mod_term);
    exe.root_module.addImport("mod_sys", mod_sys);
    exe.root_module.addImport("mod_argparse", mod_argparse);
    exe.root_module.addImport("mod_base64", mod_base64);
    exe.root_module.addImport("mod_csv", mod_csv);
    exe.root_module.addImport("mod_datetime", mod_datetime);
    exe.root_module.addImport("mod_errno", mod_errno);
    exe.root_module.addImport("mod_functools", mod_functools);
    exe.root_module.addImport("mod_hashlib", mod_hashlib);
    exe.root_module.addImport("mod_heapq", mod_heapq);
    exe.root_module.addImport("mod_io", mod_io);
    exe.root_module.addImport("mod_itertools", mod_itertools);
    exe.root_module.addImport("mod_logging", mod_logging);
    exe.root_module.addImport("mod_math", mod_math);
    exe.root_module.addImport("mod_ospath", mod_ospath);
    exe.root_module.addImport("mod_os", mod_os);
    exe.root_module.addImport("mod_pathlib", mod_pathlib);
    exe.root_module.addImport("mod_re", mod_re);
    exe.root_module.addImport("mod_shutil", mod_shutil);
    exe.root_module.addImport("mod_signal", mod_signal);
    exe.root_module.addImport("mod_statistics", mod_statistics);
    exe.root_module.addImport("mod_str_ext", mod_str_ext);
    exe.root_module.addImport("mod_strings", mod_strings);
    exe.root_module.addImport("mod_struct", mod_struct);
    exe.root_module.addImport("mod_subprocess", mod_subprocess);
    exe.root_module.addImport("mod_tempfile", mod_tempfile);
    exe.root_module.addImport("mod_textwrap", mod_textwrap);
    exe.root_module.addImport("mod_typing", mod_typing);
    exe.root_module.addImport("mod_operator", mod_operator);
    exe.root_module.addImport("mod_random", mod_random);
    exe.root_module.addImport("mod_json", mod_json);
    exe.root_module.addImport("mod_time", mod_time);
    exe.root_module.addImport("mod_collections", mod_collections);
    exe.root_module.addImport("mod_configparser", mod_configparser);
    exe.root_module.addImport("mod_unittest", mod_unittest);
    exe.root_module.addImport("mod_copy", mod_copy);
    exe.root_module.addImport("mod_array", mod_array);
    exe.root_module.addImport("mod_binascii", mod_binascii);
    exe.root_module.addImport("mod_contextlib", mod_contextlib);
    exe.root_module.addImport("mod_urllib_parse", mod_urllib_parse);
    exe.root_module.addImport("mod_uuid", mod_uuid);
    exe.root_module.addImport("mod_toml", mod_toml);
    exe.root_module.addImport("mod_secrets", mod_secrets);
    exe.root_module.addImport("mod_hmac", mod_hmac);
    exe.root_module.addImport("mod_dataclasses", mod_dataclasses);
    exe.root_module.addImport("mod_gzip", mod_gzip);
    exe.root_module.addImport("mod_zipfile", mod_zipfile);
    exe.root_module.addImport("mod_tarfile", mod_tarfile);
    exe.root_module.addImport("mod_xml_etree_elementtree", mod_xml_etree_elementtree);
    exe.root_module.addImport("mod_sqlite3", mod_sqlite3);
    exe.root_module.addImport("mod_http_client", mod_http_client);
    exe.root_module.addImport("mod_template", mod_template);
    exe.root_module.addImport("mod_fetch", mod_fetch);

    // PocketPy is C11 and expects libc.
    exe.linkLibC();

    // PocketPy amalgamated sources live in `vendor/`.
    exe.root_module.addIncludePath(b.path("vendor"));
    exe.root_module.addIncludePath(sqlite_amalgamation.path("."));
    exe.root_module.addIncludePath(b.path("vendor/bearssl/inc"));

    var c_flags: std.ArrayList([]const u8) = .empty;
    defer c_flags.deinit(b.allocator);

    // Required by PocketPy.
    c_flags.append(b.allocator, "-std=c11") catch @panic("OOM");

    // PocketPy docs recommend NDEBUG for release builds (performance).
    if (optimize != .Debug) {
        c_flags.append(b.allocator, "-DNDEBUG") catch @panic("OOM");
    }

    // Helpful across platforms; safe for a POC.
    c_flags.append(b.allocator, "-fno-strict-aliasing") catch @panic("OOM");

    exe.addCSourceFile(.{
        .file = b.path("vendor/pocketpy.c"),
        .flags = c_flags.items,
    });

    // SQLite amalgamation (statically embedded) for sqlite3 module support.
    var sqlite_flags: std.ArrayList([]const u8) = .empty;
    defer sqlite_flags.deinit(b.allocator);
    sqlite_flags.append(b.allocator, "-std=c11") catch @panic("OOM");
    if (optimize != .Debug) sqlite_flags.append(b.allocator, "-DNDEBUG") catch @panic("OOM");
    sqlite_flags.append(b.allocator, "-DSQLITE_THREADSAFE=0") catch @panic("OOM");
    sqlite_flags.append(b.allocator, "-DSQLITE_DEFAULT_MEMSTATUS=0") catch @panic("OOM");
    sqlite_flags.append(b.allocator, "-DSQLITE_OMIT_LOAD_EXTENSION") catch @panic("OOM");
    sqlite_flags.append(b.allocator, "-DSQLITE_OMIT_SHARED_CACHE") catch @panic("OOM");
    sqlite_flags.append(b.allocator, "-DSQLITE_OMIT_DEPRECATED") catch @panic("OOM");
    sqlite_flags.append(b.allocator, "-DSQLITE_DQS=0") catch @panic("OOM");
    sqlite_flags.append(b.allocator, "-DSQLITE_OMIT_UTF16") catch @panic("OOM");
    sqlite_flags.append(b.allocator, "-fno-strict-aliasing") catch @panic("OOM");
    exe.addCSourceFile(.{
        .file = sqlite_amalgamation.path("sqlite3.c"),
        .flags = sqlite_flags.items,
    });

    // TinyTemplate (single-file C templating engine).
    var tinytemplate_flags: std.ArrayList([]const u8) = .empty;
    defer tinytemplate_flags.deinit(b.allocator);
    tinytemplate_flags.append(b.allocator, "-std=c11") catch @panic("OOM");
    if (optimize != .Debug) tinytemplate_flags.append(b.allocator, "-DNDEBUG") catch @panic("OOM");
    tinytemplate_flags.append(b.allocator, "-fno-strict-aliasing") catch @panic("OOM");
    exe.addCSourceFile(.{
        .file = b.path("vendor/tinytemplate.c"),
        .flags = tinytemplate_flags.items,
    });

    // BearSSL (TLS 1.2 client) for HTTPS support in fetch/http modules.
    var bearssl_flags: std.ArrayList([]const u8) = .empty;
    defer bearssl_flags.deinit(b.allocator);
    bearssl_flags.append(b.allocator, "-std=c11") catch @panic("OOM");
    if (optimize != .Debug) bearssl_flags.append(b.allocator, "-DNDEBUG") catch @panic("OOM");
    bearssl_flags.append(b.allocator, "-Ivendor/bearssl/inc") catch @panic("OOM");
    bearssl_flags.append(b.allocator, "-Ivendor/bearssl/src") catch @panic("OOM");
    bearssl_flags.append(b.allocator, "-fno-strict-aliasing") catch @panic("OOM");
    bearssl_flags.append(b.allocator, "-ffunction-sections") catch @panic("OOM");
    bearssl_flags.append(b.allocator, "-fdata-sections") catch @panic("OOM");

    const bearssl_src_dir = std.fs.cwd().openDir("vendor/bearssl/src", .{ .iterate = true }) catch @panic("missing vendor/bearssl/src");
    var walker = bearssl_src_dir.walk(b.allocator) catch @panic("OOM");
    defer walker.deinit();
    while (walker.next() catch @panic("walk failed")) |entry| {
        if (entry.kind != .file) continue;
        if (!std.mem.endsWith(u8, entry.path, ".c")) continue;
        // These files depend on obsolete types (#if 0 in inner.h) and are not
        // part of the upstream library build.
        if (std.mem.indexOf(u8, entry.path, "ec_prime_i31_secp") != null) continue;
        exe.addCSourceFile(.{
            .file = b.path(b.fmt("vendor/bearssl/src/{s}", .{entry.path})),
            .flags = bearssl_flags.items,
        });
    }

    // Install + `zig build run` convenience.
    b.installArtifact(exe);

    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());
    if (b.args) |args| run_cmd.addArgs(args);

    const run_step = b.step("run", "Run the PocketPy ucharm runtime");
    run_step.dependOn(&run_cmd.step);
}
