use std::{env, path::PathBuf, process::Command};

fn main() {
    let native = "src/commands/account_login";
    match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => {
            let source = format!("{native}/device_windows.cpp");
            println!("cargo:rerun-if-changed={source}");
            cc::Build::new()
                .cpp(true)
                // C++/WinRT uses the standard coroutine header in C++20.
                .std("c++20")
                .flag("/EHsc")
                .file(source)
                .compile("device_login");
            println!("cargo:rustc-link-lib=windowsapp");
        }
        "macos" => {
            let source = format!("{native}/device_macos.swift");
            println!("cargo:rerun-if-changed={source}");
            let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
            let arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
                "aarch64" => "arm64",
                "x86_64" => "x86_64",
                _ => panic!("unsupported macOS architecture"),
            };
            let swift = Command::new("xcrun")
                .args(["--find", "swiftc"])
                .output()
                .unwrap();
            assert!(swift.status.success());
            let swift = PathBuf::from(String::from_utf8(swift.stdout).unwrap().trim());
            let libs = swift.parent().unwrap().join("../lib/swift/macosx");
            let sdk = Command::new("xcrun")
                .args(["--sdk", "macosx", "--show-sdk-path"])
                .output()
                .unwrap();
            assert!(sdk.status.success());
            let sdk = PathBuf::from(String::from_utf8(sdk.stdout).unwrap().trim());
            let status = Command::new(swift)
                .arg("-sdk")
                .arg(&sdk)
                .arg("-module-cache-path")
                .arg(out.join("swift-module-cache"))
                .args([
                    "-parse-as-library",
                    "-O",
                    "-emit-library",
                    "-static",
                    "-target",
                    &format!("{arch}-apple-macosx11.0"),
                    &source,
                    "-o",
                ])
                .arg(out.join("libdevice_login.a"))
                .status()
                .unwrap();
            assert!(status.success());
            println!("cargo:rustc-link-search=native={}", out.display());
            println!("cargo:rustc-link-search=native={}", libs.display());
            println!(
                "cargo:rustc-link-search=native={}",
                sdk.join("usr/lib/swift").display()
            );
            println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
            println!("cargo:rustc-link-lib=static=device_login");
        }
        _ => {}
    }
    tauri_build::build()
}
