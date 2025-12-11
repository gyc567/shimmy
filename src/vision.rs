//! Shimmy Vision Module
//!
//! Feature-gated vision capabilities for image and web analysis.
//! Mirrors Seer functionality with structured JSON output.

#[cfg(feature = "vision")]
use base64::{engine::general_purpose, Engine as _};
#[cfg(feature = "vision")]
use image::{codecs::jpeg::JpegEncoder, ColorType};
#[cfg(feature = "vision")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "vision")]
use std::time::Instant;

/// Vision response schema
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionResponse {
    pub image_path: Option<String>,
    pub url: Option<String>,
    pub mode: String,
    pub text_blocks: Vec<TextBlock>,
    pub layout: Layout,
    pub visual: Visual,
    pub interaction: Interaction,
    pub dom_map: Option<Vec<DomElement>>,
    pub meta: Meta,
    pub raw_model_output: Option<String>,
}

/// Text block from OCR
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    pub text: String,
    pub confidence: Option<f32>,
}

/// Layout analysis
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub theme: Option<String>,
    pub regions: Vec<Region>,
    pub key_ui_elements: Vec<UIElement>,
}

#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub name: String,
    pub description: String,
}

#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIElement {
    pub name: String,
    pub element_type: String,
}

/// Visual analysis
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visual {
    pub background: Option<String>,
    pub accent_colors: Vec<String>,
    pub contrast: Option<Contrast>,
    pub description: Option<String>,
}

#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contrast {
    pub ratio: Option<f32>,
    pub compliant: Option<bool>,
    pub issues: Vec<String>,
}

/// Interaction hints
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub description: Option<String>,
}

/// DOM element for web mode
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomElement {
    pub tag: String,
    pub id: Option<String>,
    pub class: Option<String>,
    pub text: Option<String>,
    pub position: Rect,
    pub attributes: std::collections::HashMap<String, String>,
}

/// Rectangle for positioning
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Metadata
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub model: String,
    pub backend: String,
    pub duration_ms: u64,
    pub parse_warnings: Option<Vec<String>>,
}

/// Vision request for HTTP API
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Deserialize)]
pub struct VisionRequest {
    pub image_base64: Option<String>,
    pub url: Option<String>,
    pub mode: String,
    pub model: Option<String>,
    #[allow(dead_code)]
    pub timeout_ms: Option<u64>,
    #[allow(dead_code)]
    pub raw: Option<bool>,
    pub license: Option<String>,
}

/// Image preprocessing configuration
#[cfg(feature = "vision")]
struct PreprocessConfig {
    max_long_edge: u32,
    max_pixels: u64,
    jpeg_quality: u8,
}

/// Preprocessed image payload passed to mtmd/vision backend
#[cfg(feature = "vision")]
struct PreprocessedImage {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
}

/// Stub implementation - returns feature disabled error
#[cfg(not(feature = "vision"))]
pub fn handle_vision_request(
    _req: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Err("Vision feature not enabled".into())
}

/// Real implementation placeholder
#[cfg(feature = "vision")]
#[allow(dead_code)]
pub fn handle_vision_request(
    _req: VisionRequest,
) -> Result<VisionResponse, Box<dyn std::error::Error>> {
    // TODO: Implement actual vision processing
    Err("Vision processing not yet implemented".into())
}

/// Process vision request with actual model inference
#[cfg(feature = "vision")]
pub async fn process_vision_request(
    req: VisionRequest,
    model_name: &str,
    license_manager: &crate::vision_license::VisionLicenseManager,
    state: &crate::AppState,
) -> Result<VisionResponse, Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    // Check license first (bypass in dev mode)
    if std::env::var("SHIMMY_VISION_DEV_MODE").is_err() {
        license_manager
            .check_vision_access(req.license.as_deref())
            .await?;
    }

    // Record usage (skip in dev mode)
    if std::env::var("SHIMMY_VISION_DEV_MODE").is_err() {
        license_manager.record_usage().await?;
    }

    // Load image data
    let raw_image_data = if let Some(base64) = &req.image_base64 {
        // Decode base64 image
        general_purpose::STANDARD
            .decode(base64)
            .map_err(|e| format!("Failed to decode base64 image: {}", e))?
    } else if let Some(url) = &req.url {
        // Fetch image from URL
        fetch_image_from_url(url).await?
    } else {
        return Err("Either image_base64 or url must be provided".into());
    };

    // Preprocess image to a safe size/format for the vision backend
    let preprocess_cfg = PreprocessConfig {
        max_long_edge: 640,
        max_pixels: 1_500_000,
        jpeg_quality: 80,
    };
    let preprocessed = preprocess_image(&raw_image_data, &preprocess_cfg)
        .map_err(|e| format!("Failed to preprocess image: {}", e))?;

    // Determine model to use (use provided model_name, fallback to env var, then default)
    let vision_model = model_name.to_string();

    // Normalize model name for registry lookup (replace : with / in registry paths)
    let registry_model_name = vision_model.replace(':', "/");

    // Check if model exists in Ollama and prompt download if needed
    if !check_ollama_model_exists(&vision_model) {
        return Err(format!(
            "Vision model '{}' is not available in Ollama.\n\
            \nTo download the default MiniCPM-V model, run:\n\
            \tollama pull registry.ollama.ai/library/minicpm-v:latest\n\
            \nOr specify a different model with --model flag or SHIMMY_VISION_MODEL environment variable.\n\
            \nAvailable vision models you can try:\n\
            \t• minicpm-v:latest (recommended default)\n\
            \t• llava:latest\n\
            \t• llava-phi3:latest\n\
            \t• moondream:latest\n\
            \t• llama3.2-vision:latest",
            vision_model
        ).into());
    }

    // Load vision model using the normalized name for registry lookup
    let model_spec = state
        .registry
        .to_spec(&registry_model_name)
        .ok_or_else(|| {
            format!(
                "Vision model '{}' not found in registry. This may be a configuration issue.",
                registry_model_name
            )
        })?;

    let loaded_model = state
        .engine
        .load(&model_spec)
        .await
        .map_err(|e| format!("Failed to load vision model: {}", e))?;

    // Prepare vision prompt based on mode
    // Encode resized image to base64 as JPEG for compression
    let image_base64 = general_purpose::STANDARD.encode(&preprocessed.bytes);
    let prompt = prepare_vision_prompt(
        &req.mode,
        preprocessed.width,
        preprocessed.height,
        &vision_model,
    );

    // Debug: print prompt length
    eprintln!(
        "DEBUG: Prompt length: {} characters, image base64 length: {} characters (not embedded in prompt)",
        prompt.len(),
        image_base64.len()
    );

    eprintln!(
        "DEBUG: About to call generate with prompt: {}",
        &prompt[..200]
    );

    // Run inference
    let gen_options = crate::engine::GenOptions {
        max_tokens: 1024,
        temperature: 0.1,
        top_p: 0.9,
        top_k: 40,
        repeat_penalty: 1.0,
        seed: None,
        stream: false,
        stop_tokens: vec!["</s>".to_string()],
    };

    // Run inference with timeout to avoid hanging
    let generate_future =
        loaded_model.generate_vision(&preprocessed.bytes, &prompt, gen_options, None);
    let raw_output =
        match tokio::time::timeout(std::time::Duration::from_secs(10), generate_future).await {
            Ok(result) => result.map_err(|e| format!("Vision inference failed: {}", e))?,
            Err(_) => return Err("Vision inference timed out after 10 seconds".into()),
        };

    eprintln!(
        "DEBUG: Generate completed, raw_output length: {}",
        raw_output.len()
    );

    // Parse model output into structured response
    let response = parse_vision_output(
        &raw_output,
        &req,
        model_name,
        start_time.elapsed().as_millis() as u64,
    )?;

    Ok(response)
}

/// Fetch image data from URL
#[cfg(feature = "vision")]
async fn fetch_image_from_url(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

/// Decode, downscale, and JPEG-encode an image to a backend-friendly payload.
#[cfg(feature = "vision")]
fn preprocess_image(
    data: &[u8],
    cfg: &PreprocessConfig,
) -> Result<PreprocessedImage, Box<dyn std::error::Error>> {
    let img = image::load_from_memory(data)?;
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();

    let mut target_w = w;
    let mut target_h = h;

    // Clamp by long edge first.
    if w.max(h) > cfg.max_long_edge {
        if w >= h {
            target_w = cfg.max_long_edge;
            target_h = ((h as f32 * cfg.max_long_edge as f32 / w as f32)
                .round()
                .max(1.0)) as u32;
        } else {
            target_h = cfg.max_long_edge;
            target_w = ((w as f32 * cfg.max_long_edge as f32 / h as f32)
                .round()
                .max(1.0)) as u32;
        }
    }

    // Enforce total pixel budget.
    let mut target_pixels = target_w as u64 * target_h as u64;
    if target_pixels > cfg.max_pixels {
        let scale = (cfg.max_pixels as f64 / target_pixels as f64).sqrt();
        target_w = ((target_w as f64 * scale).floor().max(1.0)) as u32;
        target_h = ((target_h as f64 * scale).floor().max(1.0)) as u32;
        target_pixels = target_w as u64 * target_h as u64;
    }

    // Resize if needed.
    let resized_rgb: image::RgbImage = if (target_w, target_h) != (w, h) {
        image::imageops::resize(
            &rgb,
            target_w,
            target_h,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        rgb
    };

    // Final guard against unexpected oversize inputs.
    if target_pixels > cfg.max_pixels {
        return Err(format!("image too large after resize ({}x{})", target_w, target_h).into());
    }

    let mut encoded = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut encoded, cfg.jpeg_quality);
    encoder.encode(resized_rgb.as_raw(), target_w, target_h, ColorType::Rgb8)?;

    Ok(PreprocessedImage {
        bytes: encoded,
        width: target_w,
        height: target_h,
    })
}

/// Prepare vision prompt based on analysis mode
#[cfg(feature = "vision")]
fn prepare_vision_prompt(mode: &str, width: u32, height: u32, model_name: &str) -> String {
    let base_instruction = format!(
        "You are an AI vision assistant. Analyze the provided image (dimensions: {}x{} pixels) and respond ONLY with a valid JSON object. Do not include any explanatory text before or after the JSON.\n\n",
        width,
        height
    );

    let analysis_task = match mode {
        "ocr" => "Extract all visible text from the image. Return JSON: {\"text_blocks\": [{\"text\": \"extracted text here\", \"confidence\": 0.95}]}",
        "layout" => "Analyze the layout and structure. Return JSON: {\"layout\": {\"regions\": [{\"name\": \"region_name\", \"description\": \"description\"}], \"key_ui_elements\": [{\"name\": \"element_name\", \"element_type\": \"type\"}]}}",
        "brief" => "Provide a brief visual description. Return JSON: {\"visual\": {\"description\": \"brief description of what you see\"}}",
        "web" => "Analyze as web page screenshot. Return JSON: {\"dom_map\": [{\"tag\": \"div\", \"text\": \"content\"}], \"interaction\": {\"description\": \"interactive elements\"}}",
        "full" | _ => "Perform comprehensive analysis. Return JSON with ALL fields: {\"text_blocks\": [...], \"layout\": {\"regions\": [...], \"key_ui_elements\": [...]}, \"visual\": {\"description\": \"...\"}, \"interaction\": {\"description\": \"...\"}}",
    };

    // Image is provided separately to the backend; keep prompt small to avoid Windows argv limits.
    if model_name.to_lowercase().contains("llava") {
        format!("<s>[INST] {}{} [/INST]", base_instruction, analysis_task)
    } else {
        format!(
            "<|im_start|>user\n{}{}<|im_end|>\n<|im_start|>assistant\n",
            base_instruction, analysis_task
        )
    }
}

/// Parse model output into structured vision response
#[cfg(feature = "vision")]
fn parse_vision_output(
    raw_output: &str,
    req: &VisionRequest,
    model_name: &str,
    duration_ms: u64,
) -> Result<VisionResponse, Box<dyn std::error::Error>> {
    // Try to parse as JSON first
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(raw_output) {
        return parse_structured_output(&parsed, req, model_name, duration_ms, raw_output);
    }

    // Fallback: extract JSON from text response
    if let Some(json_start) = raw_output.find('{') {
        if let Some(json_end) = raw_output.rfind('}') {
            let json_str = &raw_output[json_start..=json_end];
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                return parse_structured_output(&parsed, req, model_name, duration_ms, raw_output);
            }
        }
    }

    // Final fallback: create basic response from raw text
    Ok(VisionResponse {
        image_path: None,
        url: req.url.clone(),
        mode: req.mode.clone(),
        text_blocks: vec![TextBlock {
            text: raw_output.trim().to_string(),
            confidence: Some(0.5),
        }],
        layout: Layout {
            theme: None,
            regions: vec![],
            key_ui_elements: vec![],
        },
        visual: Visual {
            background: None,
            accent_colors: vec![],
            contrast: None,
            description: Some("Analysis completed".to_string()),
        },
        interaction: Interaction { description: None },
        dom_map: None,
        meta: Meta {
            model: model_name.to_string(),
            backend: "llama.cpp".to_string(),
            duration_ms,
            parse_warnings: Some(vec!["Could not parse structured output".to_string()]),
        },
        raw_model_output: Some(raw_output.to_string()),
    })
}

/// Parse structured JSON output into VisionResponse
#[cfg(feature = "vision")]
fn parse_structured_output(
    parsed: &serde_json::Value,
    req: &VisionRequest,
    model_name: &str,
    duration_ms: u64,
    raw_output: &str,
) -> Result<VisionResponse, Box<dyn std::error::Error>> {
    // Extract text blocks
    let text_blocks = parsed
        .get("text_blocks")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(TextBlock {
                        text: item.get("text")?.as_str()?.to_string(),
                        confidence: item
                            .get("confidence")
                            .and_then(|c| c.as_f64())
                            .map(|c| c as f32),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // Extract layout information
    let layout = if let Some(layout_obj) = parsed.get("layout") {
        Layout {
            theme: layout_obj
                .get("theme")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string()),
            regions: layout_obj
                .get("regions")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            Some(Region {
                                name: item.get("name")?.as_str()?.to_string(),
                                description: item.get("description")?.as_str()?.to_string(),
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            key_ui_elements: layout_obj
                .get("key_ui_elements")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            Some(UIElement {
                                name: item.get("name")?.as_str()?.to_string(),
                                element_type: item.get("element_type")?.as_str()?.to_string(),
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        }
    } else {
        Layout {
            theme: None,
            regions: vec![],
            key_ui_elements: vec![],
        }
    };

    // Extract visual information
    let visual = if let Some(visual_obj) = parsed.get("visual") {
        Visual {
            background: visual_obj
                .get("background")
                .and_then(|b| b.as_str())
                .map(|s| s.to_string()),
            accent_colors: visual_obj
                .get("accent_colors")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            contrast: visual_obj.get("contrast").and_then(|c| {
                Some(Contrast {
                    ratio: c.get("ratio").and_then(|r| r.as_f64()).map(|r| r as f32),
                    compliant: c.get("compliant").and_then(|c| c.as_bool()),
                    issues: c
                        .get("issues")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default(),
                })
            }),
            description: visual_obj
                .get("description")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string()),
        }
    } else {
        Visual {
            background: None,
            accent_colors: vec![],
            contrast: None,
            description: None,
        }
    };

    // Extract interaction information
    let interaction = Interaction {
        description: parsed
            .get("interaction")
            .and_then(|i| i.get("description"))
            .and_then(|d| d.as_str())
            .map(|s| s.to_string()),
    };

    // Extract DOM map for web mode
    let dom_map = parsed.get("dom_map").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|item| {
                Some(DomElement {
                    tag: item.get("tag")?.as_str()?.to_string(),
                    id: item
                        .get("id")
                        .and_then(|i| i.as_str())
                        .map(|s| s.to_string()),
                    class: item
                        .get("class")
                        .and_then(|c| c.as_str())
                        .map(|s| s.to_string()),
                    text: item
                        .get("text")
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string()),
                    position: item.get("position").and_then(|p| {
                        Some(Rect {
                            x: p.get("x")?.as_f64()? as f32,
                            y: p.get("y")?.as_f64()? as f32,
                            width: p.get("width")?.as_f64()? as f32,
                            height: p.get("height")?.as_f64()? as f32,
                        })
                    })?,
                    attributes: item
                        .get("attributes")
                        .and_then(|a| a.as_object())
                        .map(|obj| {
                            obj.iter()
                                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                                .collect()
                        })
                        .unwrap_or_default(),
                })
            })
            .collect::<Vec<_>>()
    });

    Ok(VisionResponse {
        image_path: None,
        url: req.url.clone(),
        mode: req.mode.clone(),
        text_blocks,
        layout,
        visual,
        interaction,
        dom_map,
        meta: Meta {
            model: model_name.to_string(),
            backend: "llama.cpp".to_string(),
            duration_ms,
            parse_warnings: None,
        },
        raw_model_output: Some(raw_output.to_string()),
    })
}

/// Check if a model exists in Ollama
#[cfg(feature = "vision")]
fn check_ollama_model_exists(model_name: &str) -> bool {
    // Extract the actual model name (remove registry prefix if present)
    let actual_model_name =
        if let Some(stripped) = model_name.strip_prefix("registry.ollama.ai/library/") {
            stripped.replace('/', ":")
        } else {
            model_name.to_string()
        };

    // Run ollama list and check if our model is in the output
    match std::process::Command::new("ollama")
        .args(&["list"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Look for the model name in the output (case-insensitive)
            stdout
                .lines()
                .skip(1) // Skip header line
                .any(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    parts
                        .first()
                        .map(|name| name.to_lowercase() == actual_model_name.to_lowercase())
                        .unwrap_or(false)
                })
        }
        _ => false,
    }
}
