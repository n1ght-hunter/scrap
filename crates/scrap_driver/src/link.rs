//! Target-driven linking of the compiled object into an executable.
//!
//! The link strategy is selected from the *target* triple's binary format
//! (not the compiler host): COFF/PE links via `lld-link`, ELF and Mach-O drive
//! the link through the system C compiler driver (`cc`/`clang`), which already
//! knows the crt and system library search paths. System library search paths
//! on Windows-MSVC are resolved the way rustc/cargo do, via
//! [`cc::windows_registry`], so we don't depend on an inherited `LIB` env var.
//!
//! Native targets (target == host) use the same tooling as before and are the
//! supported path; cross-linking is best-effort and depends on a cross-capable
//! linker being installed for the target format.

use std::path::Path;

use target_lexicon::{BinaryFormat, Triple};

/// MSVC system libraries required by Rust's std (bundled into the `scrap_rt`
/// staticlib). `kernel32.lib` is always linked and handled separately.
const WINDOWS_SYS_LIBS: &[&str] = &[
    "advapi32.lib",
    "bcrypt.lib",
    "msvcrt.lib",
    "ntdll.lib",
    "userenv.lib",
    "ws2_32.lib",
    "vcruntime.lib",
    "ucrt.lib",
];

/// Linux system libraries pulled in by Rust's std (bundled into the staticlib).
/// `-lgcc_s` provides the `_Unwind_*` personality routines std references; it is
/// standard on GNU/clang toolchains and accepted by `zig cc` for cross-linking.
const LINUX_SYS_LIBS: &[&str] = &["-lpthread", "-ldl", "-lm", "-lrt", "-lgcc_s"];

/// macOS system libraries. `libSystem` provides libc + libm + pthread.
const MACOS_SYS_LIBS: &[&str] = &["-lSystem"];

/// Link the object file into an executable for the given target triple.
pub fn link_executable(
    target: &Triple,
    crate_name: &str,
    obj_path: &Path,
    exe_path: &Path,
    rt_lib: Option<&Path>,
) -> anyhow::Result<()> {
    let _ = crate_name;
    match target.binary_format {
        BinaryFormat::Coff => link_pe(obj_path, exe_path, rt_lib),
        BinaryFormat::Elf => link_elf(obj_path, exe_path, rt_lib),
        BinaryFormat::Macho => link_macho(obj_path, exe_path, rt_lib),
        other => anyhow::bail!("unsupported target binary format for linking: {other:?}"),
    }
}

/// Discover MSVC/Windows SDK `LIBPATH` directories the rustc way.
///
/// Uses [`cc::windows_registry::find_tool`] to locate the MSVC toolchain and
/// reads the `LIB` entry of its environment. Falls back to the ambient `LIB`
/// env var if the tool can't be found. Returns no directories when not on a
/// Windows host targeting MSVC.
fn windows_libpaths() -> Vec<String> {
    if let Some(tool) = cc::windows_registry::find_tool("x86_64-pc-windows-msvc", "link.exe") {
        for (key, val) in tool.env() {
            if key.eq_ignore_ascii_case("LIB") {
                return std::env::split_paths(val)
                    .filter(|p| !p.as_os_str().is_empty())
                    .map(|p| p.display().to_string())
                    .collect();
            }
        }
    }

    if let Ok(lib) = std::env::var("LIB") {
        return std::env::split_paths(&lib)
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.display().to_string())
            .collect();
    }

    Vec::new()
}

/// Build the `lld-link.exe` argument vector. Pure: no env or process access, so
/// it can be unit-tested.
fn windows_link_args(
    obj_path: &Path,
    exe_path: &Path,
    rt_lib: Option<&Path>,
    libpaths: &[String],
) -> Vec<String> {
    let mut args: Vec<String> = vec![
        obj_path.to_string_lossy().into_owned(),
        "kernel32.lib".to_string(),
        "/SUBSYSTEM:CONSOLE".to_string(),
        "/ENTRY:_start".to_string(),
        format!("/OUT:{}", exe_path.display()),
    ];

    for dir in libpaths {
        args.push(format!("/LIBPATH:{dir}"));
    }

    if let Some(rt) = rt_lib {
        args.push(rt.to_string_lossy().into_owned());
        args.extend(WINDOWS_SYS_LIBS.iter().map(|s| (*s).to_string()));
    }

    args
}

fn link_pe(obj_path: &Path, exe_path: &Path, rt_lib: Option<&Path>) -> anyhow::Result<()> {
    let libpaths = windows_libpaths();
    if libpaths.is_empty() {
        anyhow::bail!(
            "could not resolve MSVC library search paths: \
             neither `cc::windows_registry` nor the `LIB` env var yielded any directories. \
             Install the MSVC build tools or run from a Developer Command Prompt."
        );
    }

    let args = windows_link_args(obj_path, exe_path, rt_lib, &libpaths);

    let status = std::process::Command::new("lld-link.exe")
        .args(&args)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run lld-link.exe: {e}"))?;

    if !status.success() {
        anyhow::bail!(
            "Linking failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

/// The C compiler driver to drive a Unix link through, as an argv vector
/// (program plus any leading args). `$CC` if set — split on whitespace so values
/// like `zig cc -target x86_64-linux-gnu` work — else `clang` for Mach-O /
/// `cc` for ELF.
fn unix_compiler(is_macho: bool) -> Vec<String> {
    if let Ok(cc) = std::env::var("CC")
        && !cc.trim().is_empty()
    {
        return cc.split_whitespace().map(str::to_string).collect();
    }
    if is_macho {
        vec!["clang".to_string()]
    } else {
        vec!["cc".to_string()]
    }
}

/// Build the argument vector passed to the C compiler driver (everything after
/// the compiler program name). Pure: no env or process access, so it can be
/// unit-tested. `isysroot` is the macOS SDK path, if any.
///
/// The link goes through the platform's crt startup (`crt1`/`Scrt1` →
/// `__libc_start_main` → `main`); codegen emits the runtime driver as `main`
/// on these targets, so we do not override the entry point or skip startfiles.
fn unix_link_args(
    obj_path: &Path,
    exe_path: &Path,
    rt_lib: Option<&Path>,
    isysroot: Option<&str>,
    sys_libs: &[&str],
) -> Vec<String> {
    let mut args: Vec<String> = vec![
        obj_path.to_string_lossy().into_owned(),
        "-o".to_string(),
        exe_path.to_string_lossy().into_owned(),
    ];

    if let Some(sdk) = isysroot {
        args.push("-isysroot".to_string());
        args.push(sdk.to_string());
    }

    if let Some(rt) = rt_lib {
        args.push(rt.to_string_lossy().into_owned());
    }

    args.extend(sys_libs.iter().map(|s| (*s).to_string()));

    args
}

/// Query the macOS SDK path via `xcrun --show-sdk-path`. Returns `None` when
/// `xcrun` is unavailable (i.e. not on a macOS host).
fn macos_sdk_path() -> Option<String> {
    let out = std::process::Command::new("xcrun")
        .args(["--show-sdk-path"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if path.is_empty() { None } else { Some(path) }
}

fn run_unix_link(compiler: &[String], args: &[String]) -> anyhow::Result<()> {
    let (program, prefix_args) = compiler
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("empty compiler command"))?;

    let status = std::process::Command::new(program)
        .args(prefix_args)
        .args(args)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run linker driver `{program}`: {e}"))?;

    if !status.success() {
        anyhow::bail!(
            "Linking failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

fn link_elf(obj_path: &Path, exe_path: &Path, rt_lib: Option<&Path>) -> anyhow::Result<()> {
    let compiler = unix_compiler(false);
    let args = unix_link_args(obj_path, exe_path, rt_lib, None, LINUX_SYS_LIBS);
    run_unix_link(&compiler, &args)
}

fn link_macho(obj_path: &Path, exe_path: &Path, rt_lib: Option<&Path>) -> anyhow::Result<()> {
    let compiler = unix_compiler(true);
    let sdk = macos_sdk_path();
    let args = unix_link_args(obj_path, exe_path, rt_lib, sdk.as_deref(), MACOS_SYS_LIBS);
    run_unix_link(&compiler, &args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn windows_args_contain_essentials() {
        let obj = PathBuf::from("target/scrap/hello.obj");
        let exe = PathBuf::from("target/scrap/hello.exe");
        let rt = PathBuf::from("target/release/scrap_rt.lib");
        let libpaths = vec![r"C:\msvc\lib".to_string(), r"C:\sdk\um\x64".to_string()];

        let args = windows_link_args(&obj, &exe, Some(&rt), &libpaths);

        assert!(args.iter().any(|a| a == "/SUBSYSTEM:CONSOLE"));
        assert!(args.iter().any(|a| a == "/ENTRY:_start"));
        assert!(args.iter().any(|a| a.starts_with("/OUT:")));
        assert!(args.iter().any(|a| a == "kernel32.lib"));
        assert!(args.iter().any(|a| a == r"/LIBPATH:C:\msvc\lib"));
        assert!(args.iter().any(|a| a == r"/LIBPATH:C:\sdk\um\x64"));
        assert!(args.iter().any(|a| a.contains("scrap_rt.lib")));
        for lib in WINDOWS_SYS_LIBS {
            assert!(args.iter().any(|a| a == lib), "missing syslib {lib}");
        }
    }

    #[test]
    fn windows_args_omit_syslibs_without_rt() {
        let obj = PathBuf::from("a.obj");
        let exe = PathBuf::from("a.exe");
        let args = windows_link_args(&obj, &exe, None, &[]);
        assert!(args.iter().any(|a| a == "kernel32.lib"));
        for lib in WINDOWS_SYS_LIBS {
            assert!(!args.iter().any(|a| a == lib), "unexpected syslib {lib}");
        }
    }

    #[test]
    fn elf_args_contain_essentials() {
        let obj = PathBuf::from("target/scrap/hello.o");
        let exe = PathBuf::from("target/scrap/hello");
        let rt = PathBuf::from("target/release/libscrap_rt.a");

        let args = unix_link_args(&obj, &exe, Some(&rt), None, LINUX_SYS_LIBS);

        assert!(args.iter().any(|a| a == "-o"));
        assert!(args.iter().any(|a| a.contains("libscrap_rt.a")));
        // Startup is driven by crt → main, so we must NOT skip startfiles or
        // override the entry point.
        assert!(!args.iter().any(|a| a == "-nostartfiles"));
        assert!(!args.iter().any(|a| a.starts_with("-Wl,-e,")));
        for lib in LINUX_SYS_LIBS {
            assert!(args.iter().any(|a| a == lib), "missing syslib {lib}");
        }
    }

    #[test]
    fn macho_args_use_macos_libs() {
        let obj = PathBuf::from("a.o");
        let exe = PathBuf::from("a");
        let args = unix_link_args(&obj, &exe, None, None, MACOS_SYS_LIBS);

        assert!(!args.iter().any(|a| a.starts_with("-Wl,-e,")));
        for lib in MACOS_SYS_LIBS {
            assert!(args.iter().any(|a| a == lib), "missing syslib {lib}");
        }
    }

    #[test]
    fn macho_isysroot_passed_through() {
        let obj = PathBuf::from("a.o");
        let exe = PathBuf::from("a");
        let args = unix_link_args(&obj, &exe, None, Some("/sdk/path"), MACOS_SYS_LIBS);
        let pos = args.iter().position(|a| a == "-isysroot").expect("isysroot");
        assert_eq!(args[pos + 1], "/sdk/path");
    }
}
