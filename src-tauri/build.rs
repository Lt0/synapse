use std::path::Path;
use std::process::Command;

fn main() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let logo_svg = workspace_root.join("VI/logo/logo.svg");
    let macos_icon_svg = workspace_root.join("VI/logo/macos-app-icon.svg");

    // Tell Cargo to rerun this script if the source SVGs or the generator itself change
    println!("cargo:rerun-if-changed={}", logo_svg.display());
    println!("cargo:rerun-if-changed={}", macos_icon_svg.display());
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root
            .join("tools/icon-generator/src/main.rs")
            .display()
    );

    // Ensure icons directory exists immediately to satisfy Cargo's initial scan
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("icons");
    if !output_dir.exists() {
        let _ = std::fs::create_dir_all(&output_dir);
    }

    // Generate icons before building
    generate_icons();

    // Build Tauri app
    tauri_build::build()
}

fn generate_icons() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let logo_svg = workspace_root.join("VI/logo/logo.svg");
    let macos_icon_svg = workspace_root.join("VI/logo/macos-app-icon.svg");
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("icons");
    let frontend_assets_dir = workspace_root.join("frontend/public");

    // Check if source files exist
    if !logo_svg.exists() {
        eprintln!(
            "cargo:warning=Logo SVG not found at {:?}, skipping icon generation",
            logo_svg
        );
        return;
    }

    if !macos_icon_svg.exists() {
        eprintln!(
            "cargo:warning=macOS icon SVG not found at {:?}, skipping icon generation",
            macos_icon_svg
        );
        return;
    }

    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        eprintln!(
            "cargo:warning=Failed to create output directory {:?}: {}",
            output_dir, e
        );
        eprintln!("cargo:warning=Continuing build anyway...");
        return;
    }

    println!("cargo:warning=Generating icons from SVG sources...");

    // Optimization: Skip icon generation if target files already exist and are newer than source SVGs
    let tray_icon = output_dir.join("tray-icon.png");
    let icon_32 = output_dir.join("32x32.png");
    let favicon = frontend_assets_dir.join("favicon.ico");

    if tray_icon.exists() && icon_32.exists() && favicon.exists() {
        let tray_meta = std::fs::metadata(&tray_icon).ok();
        let logo_meta = std::fs::metadata(&logo_svg).ok();
        let macos_meta = std::fs::metadata(&macos_icon_svg).ok();

        if let (Some(tray_time), Some(logo_time), Some(macos_time)) = (
            tray_meta.and_then(|m| m.modified().ok()),
            logo_meta.and_then(|m| m.modified().ok()),
            macos_meta.and_then(|m| m.modified().ok()),
        ) {
            if tray_time > logo_time && tray_time > macos_time {
                println!("cargo:warning=✓ Icons are up to date, skipping generation");
                return;
            }
        }
    }

    // First, ensure icon-generator is built
    let build_output = Command::new("cargo")
        .arg("build")
        .arg("--bin")
        .arg("icon-generator")
        .arg("--manifest-path")
        .arg(workspace_root.join("tools/icon-generator/Cargo.toml"))
        .arg("--release")
        .current_dir(workspace_root)
        .output();

    match build_output {
        Ok(output) if output.status.success() => {
            // Run icon-generator
            let run_output = Command::new("cargo")
                .arg("run")
                .arg("--bin")
                .arg("icon-generator")
                .arg("--manifest-path")
                .arg(workspace_root.join("tools/icon-generator/Cargo.toml"))
                .arg("--release")
                .arg("--")
                .arg("--logo-svg")
                .arg(&logo_svg)
                .arg("--macos-icon-svg")
                .arg(&macos_icon_svg)
                .arg("--output-dir")
                .arg(&output_dir)
                .arg("--frontend-assets-dir")
                .arg(&frontend_assets_dir)
                .current_dir(workspace_root)
                .output();

            match run_output {
                Ok(output) if output.status.success() => {
                    // Verify that critical icon files were actually generated
                    let tray_icon = output_dir.join("tray-icon.png");
                    let icon_32 = output_dir.join("32x32.png");

                    // Small delay to ensure file system has synced
                    std::thread::sleep(std::time::Duration::from_millis(200));

                    if !tray_icon.exists() || !icon_32.exists() {
                        eprintln!(
                            "cargo:warning=Icon generation reported success but files are missing!"
                        );
                        eprintln!("cargo:warning=Output directory: {:?}", output_dir);
                        eprintln!(
                            "cargo:warning=Expected: {:?} (exists: {})",
                            tray_icon,
                            tray_icon.exists()
                        );
                        eprintln!(
                            "cargo:warning=Expected: {:?} (exists: {})",
                            icon_32,
                            icon_32.exists()
                        );
                        if !output.stdout.is_empty() {
                            eprintln!(
                                "cargo:warning=icon-generator stdout: {}",
                                String::from_utf8_lossy(&output.stdout)
                            );
                        }
                        if !output.stderr.is_empty() {
                            eprintln!(
                                "cargo:warning=icon-generator stderr: {}",
                                String::from_utf8_lossy(&output.stderr)
                            );
                        }
                        eprintln!("cargo:warning=This may cause build failures. Please check icon-generator output.");
                    } else {
                        println!("cargo:warning=✓ Icons generated successfully");
                    }
                }
                Ok(output) => {
                    eprintln!(
                        "cargo:warning=Icon generation failed with exit code: {:?}",
                        output.status.code()
                    );
                    if !output.stderr.is_empty() {
                        eprintln!(
                            "cargo:warning=icon-generator stderr: {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                    eprintln!("cargo:warning=Continuing build anyway...");
                }
                Err(e) => {
                    eprintln!("cargo:warning=Failed to run icon-generator: {}", e);
                    eprintln!("cargo:warning=Continuing build anyway...");
                }
            }
        }
        Ok(output) => {
            eprintln!(
                "cargo:warning=Failed to build icon-generator with exit code: {:?}",
                output.status.code()
            );
            if !output.stderr.is_empty() {
                eprintln!(
                    "cargo:warning=Build stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            eprintln!("cargo:warning=Continuing build anyway...");
        }
        Err(e) => {
            eprintln!("cargo:warning=Failed to build icon-generator: {}", e);
            eprintln!("cargo:warning=Continuing build anyway...");
        }
    }
}
