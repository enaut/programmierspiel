use std::path::{Path, PathBuf};

struct ConvertTask {
    candidates: &'static [&'static str],
    outputs: &'static [&'static str],
    target_height: u32,
}

const TASKS: &[ConvertTask] = &[
    ConvertTask {
        candidates: &["mega-station-pads.jpg", "mega-station-pads.jpeg"],
        outputs: &["mega-station-pads.png"],
        target_height: 373,
    },
    ConvertTask {
        candidates: &[
            "mega-station-rings.jpg",
            "mega-station-ring.jpg",
            "mega-station-rings.jpeg",
        ],
        outputs: &["mega-station-rings.png", "mega-station-ring.png"],
        target_height: 373,
    },
];

fn find_input_file(candidates: &[&str], search_dirs: &[PathBuf]) -> Option<PathBuf> {
    for dir in search_dirs {
        for candidate in candidates {
            let path = dir.join(candidate);
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

fn needs_rebuild(input: &Path, outputs: &[PathBuf]) -> bool {
    let Ok(in_meta) = input.metadata() else {
        return false;
    };
    let Ok(in_time) = in_meta.modified() else {
        return true;
    };

    for out in outputs {
        if !out.exists() {
            return true;
        }
        let Ok(out_meta) = out.metadata() else {
            return true;
        };
        let Ok(out_time) = out_meta.modified() else {
            return true;
        };
        if out_time < in_time {
            return true;
        }
    }

    false
}

fn convert_image(
    input_path: &Path,
    output_paths: &[PathBuf],
    target_height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(input_path)?.to_rgb8();
    let (width, height) = img.dimensions();

    // 1. Process alpha transparency and color de-fringing at high resolution
    let mut rgba = image::RgbaImage::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels() {
        let r = pixel[0];
        let g = pixel[1];
        let b = pixel[2];

        // Studio white background: channels are high (r, g, b > 240)
        let min_c = r.min(g).min(b);

        // Alpha falloff:
        // min_c >= 248 is pure background (alpha = 0)
        // min_c <= 230 is pure foreground (alpha = 255)
        let alpha: u8 = if min_c >= 248 {
            0
        } else if min_c <= 230 {
            255
        } else {
            let factor = (248 - min_c) as f32 / (248 - 230) as f32;
            (factor * 255.0).round() as u8
        };

        if alpha == 0 {
            rgba.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
        } else if alpha == 255 {
            rgba.put_pixel(x, y, image::Rgba([r, g, b, 255]));
        } else {
            // De-fringe: unmix white background from semi-transparent edge
            // C = alpha * F + (1 - alpha) * 255 => F = (C - (1 - alpha) * 255) / alpha
            let alpha_f = alpha as f32 / 255.0;
            let unmix = |c: u8| -> u8 {
                let f = (c as f32 - (1.0 - alpha_f) * 255.0) / alpha_f;
                f.clamp(0.0, 255.0).round() as u8
            };
            rgba.put_pixel(x, y, image::Rgba([unmix(r), unmix(g), unmix(b), alpha]));
        }
    }

    // 2. Downscale with high-quality Lanczos3 filter
    let target_width = ((width as f64) * (target_height as f64) / (height as f64)).round() as u32;
    let resized = image::imageops::resize(
        &rgba,
        target_width,
        target_height,
        image::imageops::FilterType::Lanczos3,
    );

    // 3. Save to output paths
    for out_path in output_paths {
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        resized.save(out_path)?;
    }

    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let local_assets = manifest_dir.join("assets");
    let workspace_assets = manifest_dir.join("../assets");

    let search_dirs = [local_assets.clone(), workspace_assets];

    std::fs::create_dir_all(&local_assets).ok();

    for task in TASKS {
        if let Some(input_path) = find_input_file(task.candidates, &search_dirs) {
            println!("cargo:rerun-if-changed={}", input_path.display());

            let output_paths: Vec<PathBuf> = task
                .outputs
                .iter()
                .map(|name| local_assets.join(name))
                .collect();

            if needs_rebuild(&input_path, &output_paths) {
                if let Err(err) = convert_image(&input_path, &output_paths, task.target_height) {
                    println!(
                        "cargo:warning=Failed to convert {} to {:?}: {}",
                        input_path.display(),
                        task.outputs,
                        err
                    );
                }
            }
        }
    }
}
