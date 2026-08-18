use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use gpui::{
    App, Asset as _, AssetLogger, Context, ImageAssetLoader, ImageCacheError, RenderImage,
    Resource, SharedUri,
};

pub const DEFAULT_ARTWORK_CACHE_CAPACITY: usize = 128;

enum ArtworkEntry {
    Loading(u64),
    Ready(Arc<RenderImage>),
    Failed,
}

#[derive(Default)]
struct LruOrder(VecDeque<String>);

impl LruOrder {
    fn touch(&mut self, key: &str) {
        if let Some(index) = self.0.iter().position(|candidate| candidate == key) {
            self.0.remove(index);
        }
        self.0.push_back(key.to_owned());
    }

    fn pop_lru(&mut self) -> Option<String> {
        self.0.pop_front()
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}

/// A decoded-image cache owned by the desktop UI.
///
/// GPUI performs the network request and image decoding on its background
/// executor. Aurelia bounds the retained decoded images and controls when a
/// newly available image enters the element tree, which makes fade-in
/// transitions deterministic.
pub struct ArtworkCache {
    entries: HashMap<String, ArtworkEntry>,
    lru: LruOrder,
    capacity: usize,
    next_request_id: u64,
}

impl ArtworkCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            lru: LruOrder::default(),
            capacity: capacity.max(1),
            next_request_id: 0,
        }
    }

    pub fn get_or_load(&mut self, url: String, cx: &mut Context<Self>) -> Option<Arc<RenderImage>> {
        if let Some(entry) = self.entries.get(&url) {
            let image = match entry {
                ArtworkEntry::Ready(image) => Some(image.clone()),
                ArtworkEntry::Loading(_) | ArtworkEntry::Failed => None,
            };
            self.lru.touch(&url);
            return image;
        }

        self.next_request_id = self.next_request_id.wrapping_add(1);
        let request_id = self.next_request_id;
        self.entries
            .insert(url.clone(), ArtworkEntry::Loading(request_id));
        self.lru.touch(&url);
        self.evict_to_capacity(cx);

        let resource = Resource::Uri(SharedUri::from(url.clone()));
        let load = AssetLogger::<ImageAssetLoader>::load(resource, cx);
        let task = cx.background_executor().spawn(load);
        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                this.finish_request(url, request_id, result, cx);
                cx.notify();
            })
            .ok();
        })
        .detach();

        None
    }

    pub fn clear(&mut self, cx: &mut App) {
        for (_, entry) in self.entries.drain() {
            if let ArtworkEntry::Ready(image) = entry {
                cx.drop_image(image, None);
            }
        }
        self.lru.clear();
    }

    fn finish_request(
        &mut self,
        url: String,
        request_id: u64,
        result: Result<Arc<RenderImage>, ImageCacheError>,
        cx: &mut App,
    ) {
        let is_current = matches!(
            self.entries.get(&url),
            Some(ArtworkEntry::Loading(current_id)) if *current_id == request_id
        );
        if !is_current {
            if let Ok(image) = result {
                cx.drop_image(image, None);
            }
            return;
        }

        self.entries.insert(
            url,
            match result {
                Ok(image) => ArtworkEntry::Ready(image),
                Err(_) => ArtworkEntry::Failed,
            },
        );
    }

    fn evict_to_capacity(&mut self, cx: &mut App) {
        while self.entries.len() > self.capacity {
            let Some(key) = self.lru.pop_lru() else {
                break;
            };
            if let Some(ArtworkEntry::Ready(image)) = self.entries.remove(&key) {
                cx.drop_image(image, None);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LruOrder;

    #[test]
    fn touching_an_entry_moves_it_to_the_most_recent_end() {
        let mut order = LruOrder::default();
        order.touch("first");
        order.touch("second");
        order.touch("first");

        assert_eq!(order.pop_lru().as_deref(), Some("second"));
        assert_eq!(order.pop_lru().as_deref(), Some("first"));
    }
}
