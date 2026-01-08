use anyhow::{Context, Result};
use clap::Parser;
use image::{ImageBuffer, Rgba, RgbaImage};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "icon-generator")]
#[command(about = "Generate platform-specific icons from SVG sources")]
struct Args {
    /// Source logo SVG file (for general use)
    #[arg(long, default_value = "VI/logo/logo.svg")]
    logo_svg: PathBuf,

    /// Source macOS app icon SVG file (with background plate)
    #[arg(long, default_value = "VI/logo/macos-app-icon.svg")]
    macos_icon_svg: PathBuf,

    /// Output directory for Tauri generated icons
    #[arg(long, default_value = "src-tauri/icons")]
    output_dir: PathBuf,

    /// Output directory for Frontend assets
    #[arg(long, default_value = "frontend/public")]
    frontend_assets_dir: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Ensure output directories exist
    fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("Failed to create output directory: {:?}", args.output_dir))?;
    fs::create_dir_all(&args.frontend_assets_dir).with_context(|| {
        format!(
            "Failed to create frontend assets directory: {:?}",
            args.frontend_assets_dir
        )
    })?;

    println!("Generating icons from SVG sources...");
    println!("  Logo SVG: {:?}", args.logo_svg);
    println!("  macOS Icon SVG: {:?}", args.macos_icon_svg);
    println!("  Tauri output directory: {:?}", args.output_dir);
    println!("  Frontend output directory: {:?}", args.frontend_assets_dir);

    // Generate icons for different platforms
    // 桌面版 (Windows/Linux/macOS) 和 iOS 使用带背景板的 macos-app-icon.svg
    // 遵循 Apple HIG，包含精致的深色渐变底盘
    generate_linux_icons(&args.macos_icon_svg, &args.output_dir)?;
    generate_windows_icons(&args.macos_icon_svg, &args.output_dir)?;
    // Only generate macOS icons if iconutil is available (typically only on macOS)
    // On other platforms, this will be skipped gracefully
    if std::process::Command::new("iconutil").arg("--version").output().is_ok() {
        if let Err(e) = generate_macos_icons(&args.macos_icon_svg, &args.output_dir) {
            println!("  Warning: Failed to generate macOS icons: {}. This is expected on non-macOS platforms.", e);
        }
    } else {
        println!("  Skipping macOS icon generation (iconutil not available - this is expected on non-macOS platforms)");
    }
    generate_ios_icons(&args.macos_icon_svg, &args.output_dir)?;
    // Tray icons: macOS uses transparent logo.svg with white lines, others use colored version
    generate_tray_icon(&args.logo_svg, &args.macos_icon_svg, &args.output_dir)?;

    // Android 使用 logo.svg 作为前景（透明背景），背景使用深色底色
    // 支持 Android 自适应图标系统
    generate_android_icons(&args.logo_svg, &args.output_dir)?;

    // 为前端生成图标
    generate_frontend_icons(&args.logo_svg, &args.frontend_assets_dir)?;

    println!("✓ All icons generated successfully!");

    Ok(())
}

fn render_svg_to_png(svg_path: &Path, width: u32, height: u32) -> Result<RgbaImage> {
    let svg_data =
        fs::read(svg_path).with_context(|| format!("Failed to read SVG file: {:?}", svg_path))?;

    let opt = usvg::Options::default();

    let tree = usvg::Tree::from_data(&svg_data, &opt)
        .with_context(|| format!("Failed to parse SVG: {:?}", svg_path))?;

    let mut pixmap =
        Pixmap::new(width, height).ok_or_else(|| anyhow::anyhow!("Failed to create pixmap"))?;

    // Default behavior: center the SVG with transparent background
    let svg_size = tree.size();
    let scale_x = width as f32 / svg_size.width();
    let scale_y = height as f32 / svg_size.height();
    let scale = scale_x.min(scale_y);

    let scaled_width = (svg_size.width() * scale) as u32;
    let scaled_height = (svg_size.height() * scale) as u32;

    // Fill with transparent background
    pixmap.fill(resvg::tiny_skia::Color::TRANSPARENT);

    // Calculate transform to center the scaled SVG
    let dx = (width - scaled_width) as f32 / 2.0;
    let dy = (height - scaled_height) as f32 / 2.0;
    let transform = Transform::from_scale(scale, scale).post_translate(dx, dy);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert Pixmap to RgbaImage
    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(
        pixmap.width(),
        pixmap.height(),
        pixmap.data().to_vec(),
    )
    .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;

    Ok(img)
}

/// Apply morphological dilation to thicken lines in the image
fn dilate_image(img: &RgbaImage, radius: u32) -> RgbaImage {
    let width = img.width();
    let height = img.height();
    let mut result = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);
    
    // For each pixel in the result
    for y in 0..height {
        for x in 0..width {
            let mut max_alpha = 0u8;
            
            // Check all pixels within radius
            for dy in -(radius as i32)..=(radius as i32) {
                for dx in -(radius as i32)..=(radius as i32) {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    
                    // Check bounds
                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let pixel = img.get_pixel(nx as u32, ny as u32);
                        if pixel[3] > max_alpha {
                            max_alpha = pixel[3];
                        }
                    }
                }
            }
            
            // Set pixel to white with the maximum alpha found in the neighborhood
            if max_alpha > 0 {
                result.put_pixel(x, y, Rgba([255, 255, 255, max_alpha]));
            } else {
                result.put_pixel(x, y, Rgba([0, 0, 0, 0])); // Transparent
            }
        }
    }
    
    result
}

/// Render SVG to PNG with all colored pixels converted to white and thicker strokes (for macOS tray icons)
/// Uses morphological dilation to thicken lines after rendering
fn render_svg_to_white_png(svg_path: &Path, width: u32, height: u32) -> Result<RgbaImage> {
    // First render normally at target size
    let svg_data = fs::read(svg_path)
        .with_context(|| format!("Failed to read SVG file: {:?}", svg_path))?;
    
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(&svg_data, &opt)
        .with_context(|| format!("Failed to parse SVG: {:?}", svg_path))?;

    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| anyhow::anyhow!("Failed to create pixmap"))?;

    let svg_size = tree.size();
    let scale_x = width as f32 / svg_size.width();
    let scale_y = height as f32 / svg_size.height();
    let scale = scale_x.min(scale_y);

    let scaled_width = (svg_size.width() * scale) as u32;
    let scaled_height = (svg_size.height() * scale) as u32;

    // Fill with transparent background
    pixmap.fill(resvg::tiny_skia::Color::TRANSPARENT);

    // Calculate transform to center the scaled SVG
    let dx = (width - scaled_width) as f32 / 2.0;
    let dy = (height - scaled_height) as f32 / 2.0;
    let transform = Transform::from_scale(scale, scale).post_translate(dx, dy);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert to RgbaImage
    let mut img = ImageBuffer::<Rgba<u8>, _>::from_raw(
        pixmap.width(),
        pixmap.height(),
        pixmap.data().to_vec(),
    )
    .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;

    // Convert all non-transparent pixels to white, preserving alpha channel
    for pixel in img.pixels_mut() {
        if pixel[3] > 0 {  // If alpha > 0 (not fully transparent)
            *pixel = Rgba([255, 255, 255, pixel[3]]);  // Set RGB to white, keep alpha
        }
    }

    // Apply dilation to thicken the lines
    // For a 32x32 tray icon, use a smaller radius for subtle thickening
    let dilation_radius = if width <= 32 { 1 } else { 1 };
    let thickened = dilate_image(&img, dilation_radius);

    Ok(thickened)
}

fn save_png(img: &RgbaImage, path: &Path) -> Result<()> {
    fs::create_dir_all(path.parent().unwrap())
        .with_context(|| format!("Failed to create directory for: {:?}", path))?;
    img.save(path)
        .with_context(|| format!("Failed to save PNG: {:?}", path))?;
    Ok(())
}

fn generate_linux_icons(svg_path: &Path, output_dir: &Path) -> Result<()> {
    println!("  Generating Linux icons...");
    let sizes = [32, 64, 128, 256];
    for size in sizes {
        let img = render_svg_to_png(svg_path, size, size)?;
        let path = output_dir.join(format!("{}x{}.png", size, size));
        save_png(&img, &path)?;
    }
    // Generate 2x version for 128x128
    let img = render_svg_to_png(svg_path, 256, 256)?;
    save_png(&img, &output_dir.join("128x128@2x.png"))?;
    Ok(())
}

fn generate_windows_icons(svg_path: &Path, output_dir: &Path) -> Result<()> {
    println!("  Generating Windows icons...");
    // Windows .ico file needs multiple sizes
    let sizes = [16, 32, 48, 64, 128, 256];
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for size in sizes {
        let img = render_svg_to_png(svg_path, size, size)?;

        // Save to temporary PNG file
        let temp_png = std::env::temp_dir().join(format!("icon_{}x{}.png", size, size));
        save_png(&img, &temp_png)?;

        // Read PNG and convert to ICO entry
        let file = std::fs::File::open(&temp_png)
            .with_context(|| format!("Failed to open temp PNG: {:?}", temp_png))?;
        let ico_image = ico::IconImage::read_png(file)
            .with_context(|| format!("Failed to read PNG as ICO image: {:?}", temp_png))?;
        let entry = ico::IconDirEntry::encode(&ico_image)
            .with_context(|| format!("Failed to encode ICO entry for size {}", size))?;
        icon_dir.add_entry(entry);

        // Clean up temp file
        let _ = fs::remove_file(&temp_png);
    }

    let ico_path = output_dir.join("icon.ico");
    let mut file = fs::File::create(&ico_path)
        .with_context(|| format!("Failed to create ICO file: {:?}", ico_path))?;
    icon_dir
        .write(&mut file)
        .with_context(|| format!("Failed to write ICO file: {:?}", ico_path))?;

    Ok(())
}

fn generate_macos_icons(svg_path: &Path, output_dir: &Path) -> Result<()> {
    println!("  Generating macOS icons...");

    // macOS .icns generation using iconutil
    let iconset_dir = std::env::temp_dir().join("app.iconset");
    if iconset_dir.exists() {
        let _ = fs::remove_dir_all(&iconset_dir);
    }
    fs::create_dir_all(&iconset_dir)?;

    let sizes = [
        ("icon_16x16.png", 16),
        ("icon_16x16@2x.png", 32),
        ("icon_32x32.png", 32),
        ("icon_32x32@2x.png", 64),
        ("icon_128x128.png", 128),
        ("icon_128x128@2x.png", 256),
        ("icon_256x256.png", 256),
        ("icon_256x256@2x.png", 512),
        ("icon_512x512.png", 512),
        ("icon_512x512@2x.png", 1024),
    ];

    for (name, size) in sizes {
        let img = render_svg_to_png(svg_path, size, size)?;
        save_png(&img, &iconset_dir.join(name))?;
    }

    let icns_path = output_dir.join("icon.icns");
    let status = std::process::Command::new("iconutil")
        .arg("-c")
        .arg("icns")
        .arg(&iconset_dir)
        .arg("-o")
        .arg(&icns_path)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("    Generated icon.icns using iconutil");
        }
        _ => {
            println!("    Warning: iconutil failed or not available. Using PNG fallback.");
            let img = render_svg_to_png(svg_path, 512, 512)?;
            save_png(&img, &output_dir.join("icon.png"))?;
        }
    }

    let _ = fs::remove_dir_all(&iconset_dir);

    Ok(())
}

fn generate_android_icons(svg_path: &Path, output_dir: &Path) -> Result<()> {
    println!("  Generating Android icons...");

    let android_dir = output_dir.join("android");
    fs::create_dir_all(&android_dir)?;

    let densities = [
        ("mdpi", 48),
        ("hdpi", 72),
        ("xhdpi", 96),
        ("xxhdpi", 144),
        ("xxxhdpi", 192),
    ];

    for (density, size) in densities {
        let mipmap_dir = android_dir.join(format!("mipmap-{}", density));
        fs::create_dir_all(&mipmap_dir)?;

        let img = render_svg_to_png(svg_path, size, size)?;
        save_png(&img, &mipmap_dir.join("ic_launcher_foreground.png"))?;
        save_png(&img, &mipmap_dir.join("ic_launcher.png"))?;
        save_png(&img, &mipmap_dir.join("ic_launcher_round.png"))?;
    }

    let anydpi_dir = android_dir.join("mipmap-anydpi-v26");
    fs::create_dir_all(&anydpi_dir)?;
    let xml_content = r#"<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@color/ic_launcher_background"/>
    <foreground android:drawable="@mipmap/ic_launcher_foreground"/>
</adaptive-icon>"#;
    fs::write(anydpi_dir.join("ic_launcher.xml"), xml_content)?;

    let values_dir = android_dir.join("values");
    fs::create_dir_all(&values_dir)?;
    let bg_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <color name="ic_launcher_background">#1C1C1E</color>
</resources>"#;
    fs::write(values_dir.join("ic_launcher_background.xml"), bg_xml)?;

    Ok(())
}

fn generate_ios_icons(svg_path: &Path, output_dir: &Path) -> Result<()> {
    println!("  Generating iOS icons...");

    let ios_dir = output_dir.join("ios");
    fs::create_dir_all(&ios_dir)?;

    let ios_sizes = [
        ("AppIcon-20x20@1x.png", 20),
        ("AppIcon-20x20@2x.png", 40),
        ("AppIcon-20x20@2x-1.png", 40),
        ("AppIcon-20x20@3x.png", 60),
        ("AppIcon-29x29@1x.png", 29),
        ("AppIcon-29x29@2x.png", 58),
        ("AppIcon-29x29@2x-1.png", 58),
        ("AppIcon-29x29@3x.png", 87),
        ("AppIcon-40x40@1x.png", 40),
        ("AppIcon-40x40@2x.png", 80),
        ("AppIcon-40x40@2x-1.png", 80),
        ("AppIcon-40x40@3x.png", 120),
        ("AppIcon-60x60@2x.png", 120),
        ("AppIcon-60x60@3x.png", 180),
        ("AppIcon-76x76@1x.png", 76),
        ("AppIcon-76x76@2x.png", 152),
        ("AppIcon-83.5x83.5@2x.png", 167),
        ("AppIcon-512@2x.png", 1024),
    ];

    for (filename, size) in ios_sizes {
        let img = render_svg_to_png(svg_path, size, size)?;
        save_png(&img, &ios_dir.join(filename))?;
    }

    Ok(())
}

fn generate_tray_icon(logo_svg: &Path, macos_icon_svg: &Path, output_dir: &Path) -> Result<()> {
    println!("  Generating tray icons...");
    
    // Generate colored tray icon for Windows/Linux
    // Windows and Linux typically use colored icons in the system tray
    // Use the full icon with background for better visibility
    let img = render_svg_to_png(macos_icon_svg, 32, 32)?;
    save_png(&img, &output_dir.join("tray-icon.png"))?;
    
    // Generate pure white tray icon for macOS
    // macOS menu bar icons should be monochrome (white) template images
    // Use transparent logo.svg (no background) with white lines and thicker strokes
    // The system automatically adjusts them for light/dark menu bar backgrounds
    let white_img = render_svg_to_white_png(logo_svg, 32, 32)?;
    save_png(&white_img, &output_dir.join("tray-icon-macos.png"))?;
    
    Ok(())
}

fn generate_frontend_icons(svg_path: &Path, assets_dir: &Path) -> Result<()> {
    println!("  Generating frontend icons...");

    // 1. favicon.ico
    let mut ico_builder = ico::IconDir::new(ico::ResourceType::Icon);
    let sizes = [16, 32, 48];
    for size in sizes {
        let img = render_svg_to_png(svg_path, size, size)?;
        let temp_png = std::env::temp_dir().join(format!("favicon_{}x{}.png", size, size));
        save_png(&img, &temp_png)?;
        let file = std::fs::File::open(&temp_png)?;
        let ico_image = ico::IconImage::read_png(file)?;
        ico_builder.add_entry(ico::IconDirEntry::encode(&ico_image)?);
        let _ = fs::remove_file(&temp_png);
    }
    let ico_path = assets_dir.join("favicon.ico");
    let mut file = fs::File::create(&ico_path)?;
    ico_builder.write(&mut file)?;

    // 2. logo.png (used for og:image etc)
    let logo_png = render_svg_to_png(svg_path, 512, 512)?;
    save_png(&logo_png, &assets_dir.join("logo.png"))?;

    // 3. logo.svg
    fs::copy(svg_path, assets_dir.join("logo.svg"))?;

    Ok(())
}
