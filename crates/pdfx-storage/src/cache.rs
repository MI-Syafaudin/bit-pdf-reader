use pdfx_core::render::RenderedBitmap;
use std::collections::VecDeque;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct PageCacheKey {
    pub document_path: String,
    pub page_number: usize,
    pub width: u32,
    pub height: u32,
}

pub struct MemoryLruCache {
    max_bytes: usize,
    current_bytes: usize,
    entries: VecDeque<(PageCacheKey, RenderedBitmap)>,
}

impl MemoryLruCache {
    pub fn new(max_mb: usize) -> Self {
        Self {
            max_bytes: max_mb * 1024 * 1024,
            current_bytes: 0,
            entries: VecDeque::new(),
        }
    }

    pub fn get(&mut self, key: &PageCacheKey) -> Option<RenderedBitmap> {
        if let Some(pos) = self.entries.iter().position(|(k, _)| k == key) {
            let (k, v) = self.entries.remove(pos)?;
            let clone = v.clone();
            self.entries.push_back((k, v));
            Some(clone)
        } else {
            None
        }
    }

    pub fn put(&mut self, key: PageCacheKey, bitmap: RenderedBitmap) {
        if let Some(pos) = self.entries.iter().position(|(k, _)| k == &key) {
            if let Some((_, old)) = self.entries.remove(pos) {
                self.current_bytes = self.current_bytes.saturating_sub(old.byte_size());
            }
        }

        let size = bitmap.byte_size();
        while self.current_bytes + size > self.max_bytes && !self.entries.is_empty() {
            if let Some((_, old)) = self.entries.pop_front() {
                self.current_bytes = self.current_bytes.saturating_sub(old.byte_size());
            }
        }

        self.current_bytes += size;
        self.entries.push_back((key, bitmap));
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_bytes = 0;
    }

    pub fn current_size_bytes(&self) -> usize {
        self.current_bytes
    }
}
