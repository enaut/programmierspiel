use std::path::{Path, PathBuf};

struct ConvertTask {
    candidates: &'static [&'static str],
    outputs: &'static [&'static str],
    target_height: u32,
}

struct PairTask {
    base_candidates: &'static [&'static str],
    cmap_candidates: &'static [&'static str],
    base_outputs: &'static [&'static str],
    mask_outputs: &'static [&'static str],
    target_size: u32,
}

struct DualPairTask {
    base_candidates: &'static [&'static str],
    cmap_candidates: &'static [&'static str],
    base_outputs: &'static [&'static str],
    mask1_outputs: &'static [&'static str],
    mask2_outputs: &'static [&'static str],
    target_size: u32,
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

const PAIR_TASKS: &[PairTask] = &[
    PairTask {
        base_candidates: &["asteroid1.jpg", "asteroid1.jpeg"],
        cmap_candidates: &["asteroid1-colormap.jpg", "asteroid1-colormap.jpeg"],
        base_outputs: &["asteroid1.png"],
        mask_outputs: &["asteroid1-mask.png"],
        target_size: 512,
    },
    PairTask {
        base_candidates: &["asteroid2.jpg", "asteroid2.jpeg"],
        cmap_candidates: &["asteroid2-colormap.jpg", "asteroid2-colormap.jpeg"],
        base_outputs: &["asteroid2.png"],
        mask_outputs: &["asteroid2-mask.png"],
        target_size: 512,
    },
];

const DUAL_PAIR_TASKS: &[DualPairTask] = &[
    DualPairTask {
        base_candidates: &["player-station.jpg", "player-station.jpeg"],
        cmap_candidates: &[
            "player-station-colormap.jpg",
            "player-station-colormap.jpeg",
        ],
        base_outputs: &["player-station.png"],
        mask1_outputs: &["player-station-mask1.png", "player-station-mask.png"],
        mask2_outputs: &["player-station-mask2.png"],
        target_size: 512,
    },
    DualPairTask {
        base_candidates: &["drone1.jpg", "drone1.jpeg"],
        cmap_candidates: &["drone1-colormap.jpg", "drone1-colormap.jpeg"],
        base_outputs: &["drone1.png"],
        mask1_outputs: &["drone1-mask1.png"],
        mask2_outputs: &["drone1-mask2.png"],
        target_size: 512,
    },
    DualPairTask {
        base_candidates: &["drone-tier2.jpg", "drone-tier2.jpeg"],
        cmap_candidates: &["drone-tier2-colormap.jpg", "drone-tier2-colormap.jpeg"],
        base_outputs: &["drone-tier2.png"],
        mask1_outputs: &["drone-tier2-mask1.png"],
        mask2_outputs: &["drone-tier2-mask2.png"],
        target_size: 512,
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

fn needs_rebuild_pair(base_input: &Path, cmap_input: &Path, outputs: &[PathBuf]) -> bool {
    let base_time = match base_input.metadata().and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return true,
    };
    let cmap_time = match cmap_input.metadata().and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return true,
    };
    let newest_input = base_time.max(cmap_time);

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
        if out_time < newest_input {
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

        let min_c = r.min(g).min(b);
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

fn convert_asset_pair(
    base_path: &Path,
    cmap_path: &Path,
    base_outputs: &[PathBuf],
    mask_outputs: &[PathBuf],
    target_size: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_img = image::open(base_path)?.to_rgb8();
    let (width, height) = base_img.dimensions();
    let cmap_img = image::open(cmap_path)?.to_rgb8();
    let (c_w, c_h) = cmap_img.dimensions();

    // 1. Process alpha transparency and color de-fringing on base image
    let mut base_rgba = image::RgbaImage::new(width, height);
    let mut base_alphas = vec![0u8; (width * height) as usize];

    for (x, y, pixel) in base_img.enumerate_pixels() {
        let r = pixel[0];
        let g = pixel[1];
        let b = pixel[2];

        let min_c = r.min(g).min(b);
        let alpha: u8 = if min_c >= 248 {
            0
        } else if min_c <= 230 {
            255
        } else {
            let factor = (248 - min_c) as f32 / (248 - 230) as f32;
            (factor * 255.0).round() as u8
        };

        base_alphas[(y * width + x) as usize] = alpha;

        if alpha == 0 {
            base_rgba.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
        } else if alpha == 255 {
            base_rgba.put_pixel(x, y, image::Rgba([r, g, b, 255]));
        } else {
            let alpha_f = alpha as f32 / 255.0;
            let unmix = |c: u8| -> u8 {
                let f = (c as f32 - (1.0 - alpha_f) * 255.0) / alpha_f;
                f.clamp(0.0, 255.0).round() as u8
            };
            base_rgba.put_pixel(x, y, image::Rgba([unmix(r), unmix(g), unmix(b), alpha]));
        }
    }

    // 2. Process colormap mask (green on black), clipped to base silhouette
    let mut mask_rgba = image::RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let base_alpha = base_alphas[(y * width + x) as usize];
            if base_alpha == 0 {
                mask_rgba.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
                continue;
            }

            let cx = ((x as f64 / width as f64) * c_w as f64).min(c_w as f64 - 1.0) as u32;
            let cy = ((y as f64 / height as f64) * c_h as f64).min(c_h as f64 - 1.0) as u32;
            let c_pixel = cmap_img.get_pixel(cx, cy);
            let g = c_pixel[1];

            let mask_alpha: u8 = if g <= 10 {
                0
            } else {
                let factor = (g as f32 - 10.0) / (200.0 - 10.0);
                let val = factor.clamp(0.0, 1.0) * (base_alpha as f32 / 255.0) * 255.0;
                val.round() as u8
            };

            if mask_alpha == 0 {
                mask_rgba.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
            } else {
                mask_rgba.put_pixel(x, y, image::Rgba([255, 255, 255, mask_alpha]));
            }
        }
    }

    // 3. Downscale both to target_size (e.g. 512x512) with Lanczos3
    let target_height = target_size;
    let target_width = ((width as f64) * (target_height as f64) / (height as f64)).round() as u32;

    let resized_base = image::imageops::resize(
        &base_rgba,
        target_width,
        target_height,
        image::imageops::FilterType::Lanczos3,
    );
    let resized_mask = image::imageops::resize(
        &mask_rgba,
        target_width,
        target_height,
        image::imageops::FilterType::Lanczos3,
    );

    // 4. Save base outputs
    for out_path in base_outputs {
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        resized_base.save(out_path)?;
    }

    // 5. Save mask outputs
    for out_path in mask_outputs {
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        resized_mask.save(out_path)?;
    }

    Ok(())
}

fn convert_dual_asset_pair(
    base_path: &Path,
    cmap_path: &Path,
    base_outputs: &[PathBuf],
    mask1_outputs: &[PathBuf],
    mask2_outputs: &[PathBuf],
    target_size: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_img = image::open(base_path)?.to_rgb8();
    let (width, height) = base_img.dimensions();
    let cmap_img = image::open(cmap_path)?.to_rgb8();
    let (c_w, c_h) = cmap_img.dimensions();

    // 1. Process alpha transparency and color de-fringing on base image
    let mut base_rgba = image::RgbaImage::new(width, height);
    let mut base_alphas = vec![0u8; (width * height) as usize];

    for (x, y, pixel) in base_img.enumerate_pixels() {
        let r = pixel[0];
        let g = pixel[1];
        let b = pixel[2];

        let min_c = r.min(g).min(b);
        let alpha: u8 = if min_c >= 248 {
            0
        } else if min_c <= 230 {
            255
        } else {
            let factor = (248 - min_c) as f32 / (248 - 230) as f32;
            (factor * 255.0).round() as u8
        };

        base_alphas[(y * width + x) as usize] = alpha;

        if alpha == 0 {
            base_rgba.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
        } else if alpha == 255 {
            base_rgba.put_pixel(x, y, image::Rgba([r, g, b, 255]));
        } else {
            let alpha_f = alpha as f32 / 255.0;
            let unmix = |c: u8| -> u8 {
                let f = (c as f32 - (1.0 - alpha_f) * 255.0) / alpha_f;
                f.clamp(0.0, 255.0).round() as u8
            };
            base_rgba.put_pixel(x, y, image::Rgba([unmix(r), unmix(g), unmix(b), alpha]));
        }
    }

    // 2. Process dual colormap masks:
    // Mask 1 (Primary Color): Red channel in colormap
    // Mask 2 (Secondary Color): Green channel in colormap
    let mut mask1_rgba = image::RgbaImage::new(width, height);
    let mut mask2_rgba = image::RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let base_alpha = base_alphas[(y * width + x) as usize];
            if base_alpha == 0 {
                mask1_rgba.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
                mask2_rgba.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
                continue;
            }

            let cx = ((x as f64 / width as f64) * c_w as f64).min(c_w as f64 - 1.0) as u32;
            let cy = ((y as f64 / height as f64) * c_h as f64).min(c_h as f64 - 1.0) as u32;
            let c_pixel = cmap_img.get_pixel(cx, cy);

            let r_val = c_pixel[0];
            let g_val = c_pixel[1];

            // Primary color (Red channel)
            let mask1_alpha: u8 = if r_val <= 10 {
                0
            } else {
                let factor = (r_val as f32 - 10.0) / (200.0 - 10.0);
                let val = factor.clamp(0.0, 1.0) * (base_alpha as f32 / 255.0) * 255.0;
                val.round() as u8
            };

            if mask1_alpha == 0 {
                mask1_rgba.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
            } else {
                mask1_rgba.put_pixel(x, y, image::Rgba([255, 255, 255, mask1_alpha]));
            }

            // Secondary color (Green channel)
            let mask2_alpha: u8 = if g_val <= 10 {
                0
            } else {
                let factor = (g_val as f32 - 10.0) / (200.0 - 10.0);
                let val = factor.clamp(0.0, 1.0) * (base_alpha as f32 / 255.0) * 255.0;
                val.round() as u8
            };

            if mask2_alpha == 0 {
                mask2_rgba.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
            } else {
                mask2_rgba.put_pixel(x, y, image::Rgba([255, 255, 255, mask2_alpha]));
            }
        }
    }

    // 3. Downscale all to target_size (e.g. 512x512) with Lanczos3
    let target_height = target_size;
    let target_width = ((width as f64) * (target_height as f64) / (height as f64)).round() as u32;

    let resized_base = image::imageops::resize(
        &base_rgba,
        target_width,
        target_height,
        image::imageops::FilterType::Lanczos3,
    );
    let resized_mask1 = image::imageops::resize(
        &mask1_rgba,
        target_width,
        target_height,
        image::imageops::FilterType::Lanczos3,
    );
    let resized_mask2 = image::imageops::resize(
        &mask2_rgba,
        target_width,
        target_height,
        image::imageops::FilterType::Lanczos3,
    );

    // 4. Save base outputs
    for out_path in base_outputs {
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        resized_base.save(out_path)?;
    }

    // 5. Save mask1 outputs
    for out_path in mask1_outputs {
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        resized_mask1.save(out_path)?;
    }

    // 6. Save mask2 outputs
    for out_path in mask2_outputs {
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        resized_mask2.save(out_path)?;
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

    for task in PAIR_TASKS {
        if let (Some(base_path), Some(cmap_path)) = (
            find_input_file(task.base_candidates, &search_dirs),
            find_input_file(task.cmap_candidates, &search_dirs),
        ) {
            println!("cargo:rerun-if-changed={}", base_path.display());
            println!("cargo:rerun-if-changed={}", cmap_path.display());

            let base_outputs: Vec<PathBuf> = task
                .base_outputs
                .iter()
                .map(|name| local_assets.join(name))
                .collect();
            let mask_outputs: Vec<PathBuf> = task
                .mask_outputs
                .iter()
                .map(|name| local_assets.join(name))
                .collect();

            let mut all_outputs = base_outputs.clone();
            all_outputs.extend(mask_outputs.clone());

            if needs_rebuild_pair(&base_path, &cmap_path, &all_outputs) {
                if let Err(err) = convert_asset_pair(
                    &base_path,
                    &cmap_path,
                    &base_outputs,
                    &mask_outputs,
                    task.target_size,
                ) {
                    println!(
                        "cargo:warning=Failed to convert pair {} + {} to {:?}: {}",
                        base_path.display(),
                        cmap_path.display(),
                        task.base_outputs,
                        err
                    );
                }
            }
        }
    }

    for task in DUAL_PAIR_TASKS {
        if let (Some(base_path), Some(cmap_path)) = (
            find_input_file(task.base_candidates, &search_dirs),
            find_input_file(task.cmap_candidates, &search_dirs),
        ) {
            println!("cargo:rerun-if-changed={}", base_path.display());
            println!("cargo:rerun-if-changed={}", cmap_path.display());

            let base_outputs: Vec<PathBuf> = task
                .base_outputs
                .iter()
                .map(|name| local_assets.join(name))
                .collect();
            let mask1_outputs: Vec<PathBuf> = task
                .mask1_outputs
                .iter()
                .map(|name| local_assets.join(name))
                .collect();
            let mask2_outputs: Vec<PathBuf> = task
                .mask2_outputs
                .iter()
                .map(|name| local_assets.join(name))
                .collect();

            let mut all_outputs = base_outputs.clone();
            all_outputs.extend(mask1_outputs.clone());
            all_outputs.extend(mask2_outputs.clone());

            if needs_rebuild_pair(&base_path, &cmap_path, &all_outputs) {
                if let Err(err) = convert_dual_asset_pair(
                    &base_path,
                    &cmap_path,
                    &base_outputs,
                    &mask1_outputs,
                    &mask2_outputs,
                    task.target_size,
                ) {
                    println!(
                        "cargo:warning=Failed to convert dual pair {} + {} to {:?}: {}",
                        base_path.display(),
                        cmap_path.display(),
                        task.base_outputs,
                        err
                    );
                }
            }
        }
    }
}
