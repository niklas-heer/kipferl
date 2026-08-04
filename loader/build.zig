const std = @import("std");

pub fn build(b: *std.Build) void {
    const optimize = b.standardOptimizeOption(.{});

    // Build loader stubs for all target platforms
    const targets = [_]struct {
        cpu_arch: std.Target.Cpu.Arch,
        os_tag: std.Target.Os.Tag,
        name: []const u8,
    }{
        .{ .cpu_arch = .aarch64, .os_tag = .macos, .name = "loader-macos-aarch64" },
        .{ .cpu_arch = .x86_64, .os_tag = .macos, .name = "loader-macos-x86_64" },
        .{ .cpu_arch = .x86_64, .os_tag = .linux, .name = "loader-linux-x86_64" },
        .{ .cpu_arch = .aarch64, .os_tag = .linux, .name = "loader-linux-aarch64" },
    };

    for (targets) |target_info| {
        const target = b.resolveTargetQuery(.{
            .cpu_arch = target_info.cpu_arch,
            .os_tag = target_info.os_tag,
        });

        // Create trailer module with target
        const trailer_mod = b.createModule(.{
            .root_source_file = b.path("src/trailer.zig"),
            .target = target,
            .optimize = .ReleaseSmall,
        });

        // Create executor module with target
        const executor_mod = b.createModule(.{
            .root_source_file = b.path("src/executor.zig"),
            .target = target,
            .optimize = .ReleaseSmall,
            .imports = &.{
                .{ .name = "trailer", .module = trailer_mod },
            },
        });

        // Create main module
        const main_mod = b.createModule(.{
            .root_source_file = b.path("src/main.zig"),
            .target = target,
            .optimize = .ReleaseSmall,
            .imports = &.{
                .{ .name = "trailer", .module = trailer_mod },
                .{ .name = "executor", .module = executor_mod },
            },
        });

        const exe = b.addExecutable(.{
            .name = target_info.name,
            .root_module = main_mod,
        });

        // Strip symbols and optimize for size
        exe.root_module.strip = true;

        // Install to stubs/ directory
        const install = b.addInstallArtifact(exe, .{
            .dest_dir = .{ .override = .{ .custom = "stubs" } },
        });

        b.getInstallStep().dependOn(&install.step);
    }

    // Build for host platform only (for testing)
    const host_target = b.standardTargetOptions(.{});

    const host_trailer_mod = b.createModule(.{
        .root_source_file = b.path("src/trailer.zig"),
        .target = host_target,
        .optimize = optimize,
    });

    const host_executor_mod = b.createModule(.{
        .root_source_file = b.path("src/executor.zig"),
        .target = host_target,
        .optimize = optimize,
        .imports = &.{
            .{ .name = "trailer", .module = host_trailer_mod },
        },
    });

    const host_exe = b.addExecutable(.{
        .name = "loader",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/main.zig"),
            .target = host_target,
            .optimize = optimize,
            .imports = &.{
                .{ .name = "trailer", .module = host_trailer_mod },
                .{ .name = "executor", .module = host_executor_mod },
            },
        }),
    });
    b.installArtifact(host_exe);

    // Tests
    const test_step = b.step("test", "Run unit tests");

    const trailer_tests = b.addTest(.{
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/trailer.zig"),
            .target = host_target,
            .optimize = optimize,
        }),
    });
    const run_trailer_tests = b.addRunArtifact(trailer_tests);
    test_step.dependOn(&run_trailer_tests.step);

    const executor_tests = b.addTest(.{
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/executor.zig"),
            .target = host_target,
            .optimize = optimize,
        }),
    });
    const run_executor_tests = b.addRunArtifact(executor_tests);
    test_step.dependOn(&run_executor_tests.step);
}
