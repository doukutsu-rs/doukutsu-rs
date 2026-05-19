use std::cell::RefCell;
use std::rc::Rc;

use crate::framework::context::Context;

pub trait ClipboardBackend: 'static {
    fn get_text(&mut self) -> Option<String>;
    fn set_text(&mut self, text: &str);
}

pub struct NullClipboard;

impl ClipboardBackend for NullClipboard {
    fn get_text(&mut self) -> Option<String> {
        None
    }
    fn set_text(&mut self, _text: &str) {}
}

#[derive(Clone)]
pub struct ClipboardContext {
    inner: Rc<RefCell<dyn ClipboardBackend>>,
}

impl ClipboardContext {
    pub fn new<B: ClipboardBackend>(backend: B) -> Self {
        Self { inner: Rc::new(RefCell::new(backend)) }
    }

    pub fn get_text(&self) -> Option<String> {
        self.inner.borrow_mut().get_text()
    }

    pub fn set_text(&self, text: &str) {
        self.inner.borrow_mut().set_text(text);
    }

    pub(crate) fn imgui_adapter(&self) -> ImguiClipboardAdapter {
        ImguiClipboardAdapter { inner: self.inner.clone() }
    }
}

impl Default for ClipboardContext {
    fn default() -> Self {
        Self::new(NullClipboard)
    }
}

pub(crate) struct ImguiClipboardAdapter {
    inner: Rc<RefCell<dyn ClipboardBackend>>,
}

impl imgui::ClipboardBackend for ImguiClipboardAdapter {
    fn get(&mut self) -> Option<String> {
        self.inner.borrow_mut().get_text()
    }
    fn set(&mut self, value: &str) {
        self.inner.borrow_mut().set_text(value);
    }
}

pub fn get(ctx: &Context) -> Option<String> {
    ctx.clipboard.get_text()
}

pub fn set(ctx: &Context, text: &str) {
    ctx.clipboard.set_text(text);
}
