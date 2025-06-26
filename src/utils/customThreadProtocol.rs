//! Widget that separates resize+encode from rendering.
//! This allows for rendering to be non-blocking, offloading resize+encode into another thread.
//! See examples/async.rs for how to setup the threads and channels.
//! At least one worker thread for resize+encode is required, the example shows how to combine
//! the needs-resize-polling with other terminal events into one event loop.
#![allow(dead_code)]

use std::sync::mpsc::Sender;

use image::Rgba;
use ratatui::prelude::{Buffer, Rect};

use ratatui_image::errors::Errors;
use ratatui_image::protocol::{StatefulProtocol, StatefulProtocolType};
use ratatui_image::{Resize, ResizeEncodeRender};

/// The only usage of this struct is to call `perform()` on it and pass the completed resize to `ThreadProtocols` `update_protocol()`
pub struct CustomResizeRequest {
    image_id: usize,
    protocol: StatefulProtocol,
    resize: Resize,
    area: Rect,
    id: u64,
}

impl CustomResizeRequest {
    pub fn resize_encode(mut self) -> Result<CustomResizeResponse, Errors> {
        self.protocol.resize_encode(&self.resize, self.area);
        self.protocol
            .last_encoding_result()
            .expect("The resize has just been performed")?;
        Ok(CustomResizeResponse {
            image_id: self.image_id,
            protocol: self.protocol,
            id: self.id,
        })
    }
    pub fn image_id(&self) -> usize {
        self.image_id
    }
}

/// The only usage of this struct is to pass it to `ThreadProtocols` `update_resize_protocol()`
pub struct CustomResizeResponse {
    image_id: usize,
    protocol: StatefulProtocol,
    id: u64,
}

/// The state of a ThreadImage.
///
/// Has `inner` [StatefulProtocol] and sents requests through the mspc channel to do the
/// `resize_encode()` work.
pub struct CustomThreadProtocol {
    image_id: usize,
    inner: Option<StatefulProtocol>,
    tx: Sender<CustomResizeRequest>,
    id: u64,
}

impl CustomThreadProtocol {
    pub fn new(image_id: usize, tx: Sender<CustomResizeRequest>, inner: Option<StatefulProtocol>) -> CustomThreadProtocol {
        Self { image_id, inner, tx, id: 0 }
    }

    pub fn replace_protocol(&mut self, proto: StatefulProtocol) {
        self.inner = Some(proto);
        self.increment_id();
    }

    pub fn protocol_type(&self) -> Option<&StatefulProtocolType> {
        self.inner.as_ref().map(|inner| inner.protocol_type())
    }

    pub fn protocol_type_owned(self) -> Option<StatefulProtocolType> {
        self.inner.map(|inner| inner.protocol_type_owned())
    }

    // Get the background color that fills in when resizing.
    pub fn background_color(&self) -> Option<Rgba<u8>> {
        self.inner.as_ref().map(|inner| inner.background_color())
    }

    /// This function should be used when an image should be updated but the updated image is not yet available
    pub fn empty_protocol(&mut self) {
        self.inner = None;
        self.increment_id();
    }

    pub fn update_resized_protocol(&mut self, completed: CustomResizeResponse) -> bool {
        let equal = self.id == completed.id;
        if equal {
            self.inner = Some(completed.protocol)
        }
        equal
    }

    pub fn size_for(&self, resize: Resize, area: Rect) -> Option<Rect> {
        self.inner
            .as_ref()
            .map(|protocol| protocol.size_for(resize, area))
    }

    fn increment_id(&mut self) {
        self.id = self.id.wrapping_add(1);
    }
}

impl ResizeEncodeRender for CustomThreadProtocol {
    fn needs_resize(&self, resize: &Resize, area: Rect) -> Option<Rect> {
        self.inner
            .as_ref()
            .and_then(|protocol| protocol.needs_resize(resize, area))
    }

    /// Senda a `ResizeRequest` through the channel if there already isn't a pending `ResizeRequest`
    fn resize_encode(&mut self, resize: &Resize, area: Rect) {
        let _ = self.inner.take().map(|protocol| {
            self.increment_id();
            let _ = self.tx.send(CustomResizeRequest {
                image_id: self.image_id,
                protocol,
                resize: resize.clone(),
                area,
                id: self.id,
            });
        });
    }

    /// Render the currently resized and encoded data to the buffer, if there isn't a pending `ResizeRequest`
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let _ = self
            .inner
            .as_mut()
            .map(|protocol| protocol.render(area, buf));
    }
}

