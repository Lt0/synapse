use std::path::Path;
use std::process::Command;

fn main() {
    // Ensure assets directory exists immediately
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    if !output_dir.exists() {
        let _ = std::fs::create_dir_all(&output_dir);
    }

    // Generate icons
    generate_icons();
}

fn generate_icons() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let logo_svg = workspace_root.join("VI/logo/logo.svg");
    let macos_icon_svg = workspace_root.join("VI/logo/macos-app-icon.svg");
    let frontend_assets_dir = workspace_root.join("frontend/assets");
    let tauri_icons_dir = workspace_root.join("src-tauri/icons");

    // Check if source files exist
    if !logo_svg.exists() {
        eprintln!(
            "cargo:warning=Logo SVG not found at {:?}, skipping icon generation",
            logo_svg
        );
        return;
    }

    // Ensure frontend assets directory exists
    if let Err(e) = std::fs::create_dir_all(&frontend_assets_dir) {
        eprintln!(
            "cargo:warning=Failed to create frontend assets directory {:?}: {}",
            frontend_assets_dir, e
        );
        return;
    }

    println!("cargo:warning=Generating frontend icons from SVG sources...");

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
                .arg(&tauri_icons_dir)
                .arg("--frontend-assets-dir")
                .arg(&frontend_assets_dir)
                .current_dir(workspace_root)
                .output();

            match run_output {
                Ok(output) if output.status.success() => {
                    // Verify frontend icons
                    let favicon = frontend_assets_dir.join("favicon.ico");
                    if !favicon.exists() {
                        eprintln!("cargo:warning=Frontend icon generation reported success but favicon.ico is missing!");
                    } else {
                        println!("cargo:warning=✓ Frontend icons generated successfully");
                    }
                }
                Ok(output) => {
                    eprintln!(
                        "cargo:warning=Icon generation failed with exit code: {:?}",
                        output.status.code()
                    );
                }
                Err(e) => {
                    eprintln!("cargo:warning=Failed to run icon-generator: {}", e);
                }
            }
        }
        _ => {
            eprintln!("cargo:warning=Failed to build icon-generator");
        }
    }
}
