//! M1 verification for the GPU `VideoFrame` primitive.
//!
//! Renders a static RGBA test pattern through the new atlas-bypassing texture
//! path. If the pipeline works you should see, at full resolution:
//!   - red increasing left -> right
//!   - green increasing top -> bottom
//!   - a blue checkerboard
//! (so colour order, orientation, and scaling are all verifiable by eye).
//!
//! Run:  cargo run -p video_frame_demo

use std::sync::Arc;

use floem::prelude::*;
use floem::views::{video_frame, RgbaFrame};

fn make_test_frame(w: u32, h: u32) -> RgbaFrame {
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            data[i] = (x * 255 / w.max(1)) as u8; // R: left -> right
            data[i + 1] = (y * 255 / h.max(1)) as u8; // G: top -> bottom
            data[i + 2] = if (x / 32 + y / 32) % 2 == 0 { 200 } else { 40 }; // B checker
            data[i + 3] = 255;
        }
    }
    RgbaFrame {
        data: Arc::new(data),
        width: w,
        height: h,
    }
}

fn app_view() -> impl IntoView {
    let frame = make_test_frame(512, 512);
    video_frame(move || Some(frame.clone())).style(|s| s.size(512.0, 512.0).margin(20.0))
}

fn main() {
    floem::launch(app_view);
}
