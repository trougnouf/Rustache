// File: src/color_utils.rs
use std::hash::{Hash, Hasher};

/// Generates a deterministic color tuple (r, g, b) in [0.0, 1.0] range based on the input string.
/// Focuses on Pastel tones (High Lightness, Medium+ Saturation) for a "happy" look,
/// while maximizing distinctness for many tags.
pub fn generate_color(tag: &str) -> (f32, f32, f32) {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    tag.hash(&mut hasher);
    let hash = hasher.finish();

    // Hue: 0-360 degrees (Primary differentiator)
    let h = (hash % 360) as f32;

    // Use different bits of the hash for S and L to avoid correlation with Hue
    let hash_s = hash >> 16;
    let hash_l = hash >> 32;

    // Saturation: 40% - 90%
    // Pastels are often softer (lower S), but to distinguish 100+ tags we use a wider range
    // avoiding the very dull (<60%) and the neon-like (>90%).
    let s = 0.40 + ((hash_s % 51) as f32 / 100.0);

    // Lightness: 65% - 90%
    // This strictly enforces the "Pastel" feel.
    // 70% is light enough to support black text easily.
    // 90% is nearly white but still tinted.
    let l = 0.65 + ((hash_l % 26) as f32 / 100.0);

    hsl_to_rgb(h, s, l)
}

/// Helper: HSL to RGB conversion
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if (0.0..60.0).contains(&h) {
        (c, x, 0.0)
    } else if (60.0..120.0).contains(&h) {
        (x, c, 0.0)
    } else if (120.0..180.0).contains(&h) {
        (0.0, c, x)
    } else if (180.0..240.0).contains(&h) {
        (0.0, x, c)
    } else if (240.0..300.0).contains(&h) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r + m, g + m, b + m)
}

/// Determines if text on top of this color should be white.
/// Since we are targeting pastels (High Lightness), this will almost always be false,
/// implying we should use Black text, which looks best on pastels.
pub fn is_dark(r: f32, g: f32, b: f32) -> bool {
    let brightness = 0.299 * r + 0.587 * g + 0.114 * b;
    brightness < 0.5
}
