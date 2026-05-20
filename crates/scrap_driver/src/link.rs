//! Platform-dispatched linking of the compiled object into an executable.
//!
//! System library search paths are resolved the way rustc/cargo do:
//! on Windows-MSVC via [`cc::windows_registry`] (so we no longer depend on an
//! inherited `LIB` env var), and on Unix by driving the link through the system
//! C compiler driver (`cc`/`clang`), which already knows the crt and system
//! library search paths.

use std::path::Path;

/// MSVC system libraries required by Rust's std (bundled into the `scrap_rt`
/// staticlib). `kernel32.lib` is always linked and handled separately.
#[cfg(windows)]
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
#[cfg(all(unix, not(target_os = "macos")))]
const LINUX_SYS_LIBS: &[&str] = &["-lpthread", "-ldl", "-lm", "-lrt"];

/// macOS system libraries. `libSystem` provides libc + libm + pthread.
#[cfg(target_os = "macos")]
const MACOS_SYS_LIBS: &[&str] = &["-lSystem"];

/// Link the object file into an executable for the host platform.
pub fn link_executable(
    crate_name: &str,
    obj_path: &Path,
    exe_path: &Path,
    rt_lib: Option<&Path>,
) -> anyhow::Result<()> {
    let _ = crate_name;
    #[cfg(windows)]
    {
        link_windows(obj_path, exe_path, rt_lib)
    }
    #[cfg(unix)]
    {
        link_unix(obj_path, exe_path, rt_lib)
    }
}

/// Discover MSVC/Windows SDK `LIBPATH` directories the rustc way.
///
/// Uses [`cc::windows_registry::find_tool`] to locate the MSVC toolchain and
/// reads the `LIB` entry of its environment. Falls back to the ambient `LIB`
/// env var if the tool can't be found.
#[cfg(windows)]
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
#[cfg(windows)]
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

#[cfg(windows)]
fn link_windows(obj_path: &Path, exe_path: &Path, rt_lib: Option<&Path>) -> anyhow::Result<()> {
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

/// The C compiler driver to drive the Unix link through: `$CC` if set, else
/// `clang` on macOS / `cc` on Linux.
#[cfg(unix)]
fn unix_compiler() -> String {
    if let Ok(cc) = std::env::var("CC")
        && !cc.is_empty()
    {
        return cc;
    }
    if cfg!(target_os = "macos") {
        "clang".to_string()
    } else {
        "cc".to_string()
    }
}

/// Build the argument vector passed to the C compiler driver (everything after
/// the compiler program name). Pure: no env or process access, so it can be
/// unit-tested. `isysroot` is the macOS SDK path, if any.
#[cfg(unix)]
fn unix_link_args(
    obj_path: &Path,
    exe_path: &Path,
    rt_lib: Option<&Path>,
    isysroot: Option<&str>,
) -> Vec<String> {
    let entry = if cfg!(target_os = "macos") {
        "-Wl,-e,__start"
    } else {
        "-Wl,-e,_start"
    };

    let mut args: Vec<String> = vec![
        obj_path.to_string_lossy().into_owned(),
        "-o".to_string(),
        exe_path.to_string_lossy().into_owned(),
        "-nostartfiles".to_string(),
        entry.to_string(),
    ];

    if let Some(sdk) = isysroot {
        args.push("-isysroot".to_string());
        args.push(sdk.to_string());
    }

    if let Some(rt) = rt_lib {
        args.push(rt.to_string_lossy().into_owned());
    }

    #[cfg(target_os = "macos")]
    args.extend(MACOS_SYS_LIBS.iter().map(|s| (*s).to_string()));
    #[cfg(all(unix, not(target_os = "macos")))]
    args.extend(LINUX_SYS_LIBS.iter().map(|s| (*s).to_string()));

    args
}

/// Query the macOS SDK path via `xcrun --show-sdk-path`.
#[cfg(target_os = "macos")]
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

#[cfg(unix)]
fn link_unix(obj_path: &Path, exe_path: &Path, rt_lib: Option<&Path>) -> anyhow::Result<()> {
    let compiler = unix_compiler();

    #[cfg(target_os = "macos")]
    let sdk = macos_sdk_path();
    #[cfg(target_os = "macos")]
    let isysroot = sdk.as_deref();
    #[cfg(not(target_os = "macos"))]
    let isysroot: Option<&str> = None;

    let args = unix_link_args(obj_path, exe_path, rt_lib, isysroot);

    let status = std::process::Command::new(&compiler)
        .args(&args)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run linker driver `{compiler}`: {e}"))?;

    if !status.success() {
        anyhow::bail!(
            "Linking failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[cfg(windows)]
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

    #[cfg(windows)]
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

    #[cfg(unix)]
    #[test]
    fn unix_args_contain_essentials() {
        let obj = PathBuf::from("target/scrap/hello.o");
        let exe = PathBuf::from("target/scrap/hello");
        let rt = PathBuf::from("target/release/libscrap_rt.a");

        let args = unix_link_args(&obj, &exe, Some(&rt), None);

        assert!(args.iter().any(|a| a == "-nostartfiles"));
        assert!(args.iter().any(|a| a == "-o"));
        assert!(args.iter().any(|a| a.starts_with("-Wl,-e,")));
        assert!(args.iter().any(|a| a.contains("libscrap_rt.a")));

        #[cfg(target_os = "macos")]
        {
            assert!(args.iter().any(|a| a == "-Wl,-e,__start"));
            for lib in MACOS_SYS_LIBS {
                assert!(args.iter().any(|a| a == lib), "missing syslib {lib}");
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert!(args.iter().any(|a| a == "-Wl,-e,_start"));
            for lib in LINUX_SYS_LIBS {
                assert!(args.iter().any(|a| a == lib), "missing syslib {lib}");
            }
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_isysroot_passed_through() {
        let obj = PathBuf::from("a.o");
        let exe = PathBuf::from("a");
        let args = unix_link_args(&obj, &exe, None, Some("/sdk/path"));
        let pos = args.iter().position(|a| a == "-isysroot").expect("isysroot");
        assert_eq!(args[pos + 1], "/sdk/path");
    }
}
