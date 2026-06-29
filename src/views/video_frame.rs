use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use floem_reactive::SignalTracker;
use floem_renderer::{ExternalTexture, Renderer};

use crate::{context::PaintCx, view::View, view::ViewId};

static NEXT_TEXTURE_ID: AtomicU64 = AtomicU64::new(1);

/// A single RGBA8 frame. `data` must be `width * height * 4` bytes
/// (RGBA8, unmultiplied alpha, row 0 = top).
#[derive(Clone)]
pub struct RgbaFrame {
    pub data: Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

/// A view that displays a streaming RGBA frame directly on the GPU, bypassing
/// the renderer's image atlas. Suitable for large, high-frame-rate surfaces
/// such as a live emulator/video stream.
///
/// The `frame` closure is called (reactively) on each paint; reading any
/// signal inside it automatically schedules a repaint when the frame changes —
/// the same pattern as [`canvas`](super::canvas).
#[allow(clippy::type_complexity)]
pub struct VideoFrame {
    id: ViewId,
    texture_id: u64,
    frame_fn: Box<dyn Fn() -> Option<RgbaFrame>>,
    tracker: Option<SignalTracker>,
}

/// Creates a [`VideoFrame`] view driven by `frame`, which returns the latest
/// RGBA frame to display (or `None` to draw nothing).
///
/// # Example
/// ```ignore
/// video_frame(move || {
///     latest.get().map(|f| RgbaFrame { data: f.rgba.clone(), width: f.w, height: f.h })
/// })
/// .style(|s| s.size_full());
/// ```
pub fn video_frame(frame: impl Fn() -> Option<RgbaFrame> + 'static) -> VideoFrame {
    VideoFrame {
        id: ViewId::new(),
        texture_id: NEXT_TEXTURE_ID.fetch_add(1, Ordering::Relaxed),
        frame_fn: Box::new(frame),
        tracker: None,
    }
}

impl View for VideoFrame {
    fn id(&self) -> ViewId {
        self.id
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "VideoFrame".into()
    }

    fn paint(&mut self, cx: &mut PaintCx) {
        let id = self.id;
        let texture_id = self.texture_id;

        if self.tracker.is_none() {
            self.tracker = Some(SignalTracker::new(move || {
                id.request_paint();
            }));
        }

        let frame_fn = &self.frame_fn;
        let tracker = self.tracker.as_ref().unwrap();
        tracker.track(|| {
            if let Some(f) = frame_fn() {
                let rect = id.get_content_rect();
                cx.draw_external_texture(
                    ExternalTexture {
                        id: texture_id,
                        data: &f.data,
                        width: f.width,
                        height: f.height,
                    },
                    rect,
                );
            }
        });
    }
}
