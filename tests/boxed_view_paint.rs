use std::cell::Cell;
use std::rc::Rc;

use floem::context::PaintCx;
use floem::headless::{HeadlessHarness, TestRoot};
use floem::views::{Container, Decorators};
use floem::{AnyView, IntoView, View, ViewId};

/// A view that counts how often each paint hook runs.
struct Probe {
    id: ViewId,
    paints: Rc<Cell<usize>>,
    post_paints: Rc<Cell<usize>>,
}

impl View for Probe {
    fn id(&self) -> ViewId {
        self.id
    }

    fn paint(&mut self, _cx: &mut PaintCx) {
        self.paints.set(self.paints.get() + 1);
    }

    fn post_paint(&mut self, _cx: &mut PaintCx) {
        self.post_paints.set(self.post_paints.get() + 1);
    }
}

/// Paints a probe wrapped in `boxes` layers of `Box<dyn View>` once and
/// returns how many times `paint` and `post_paint` ran.
fn paint_counts(boxes: usize) -> (usize, usize) {
    let root = TestRoot::new();
    let paints = Rc::new(Cell::new(0));
    let post_paints = Rc::new(Cell::new(0));
    let mut view: AnyView = Probe {
        id: ViewId::new(),
        paints: paints.clone(),
        post_paints: post_paints.clone(),
    }
    .style(|s| s.size(10.0, 10.0))
    .into_any();
    for _ in 1..boxes {
        view = view.into_any();
    }
    let container = Container::new(view).style(|s| s.size(100.0, 100.0));
    let mut harness = HeadlessHarness::new_with_size(root, container, 100.0, 100.0);
    harness.rebuild();
    paints.set(0);
    post_paints.set(0);
    harness.paint();
    (paints.get(), post_paints.get())
}

#[test]
fn a_boxed_view_paints_once_and_gets_its_post_paint() {
    // `Container::new` boxes its child again, so even one explicit box
    // is two layers by the time the tree paints.
    for boxes in 1..=3 {
        assert_eq!(
            paint_counts(boxes),
            (1, 1),
            "with {boxes} explicit box layers"
        );
    }
}
