{
  lib,
  rustPlatform,
  versionCheckHook,
}:

let
  workspace = (lib.importTOML ../Cargo.toml).workspace.package;
in
rustPlatform.buildRustPackage {
  pname = "kipferl";
  inherit (workspace) version;

  src = lib.cleanSource ../.;

  cargoLock.lockFile = ../Cargo.lock;
  cargoBuildFlags = [
    "--package"
    "kipferl-cli"
  ];

  # CI runs the full suite; the package build only proves the binary starts.
  doCheck = false;
  doInstallCheck = true;
  nativeInstallCheckInputs = [ versionCheckHook ];

  meta = {
    description = "Bake Python CLI apps into fast, standalone binaries";
    homepage = workspace.repository;
    license = lib.licenses.mit;
    mainProgram = "kipferl";
    # The CLI embeds prebuilt runtimes for these hosts only.
    platforms = [
      "aarch64-darwin"
      "aarch64-linux"
      "x86_64-darwin"
      "x86_64-linux"
    ];
  };
}
