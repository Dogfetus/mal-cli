#![allow(unused)]
//TODO: use the one thread per image approach again
//TODO: fix these doc strings
//
//! claude used to generate doc strings lol
//! Image management utilities for terminal-based anime applications.
//!
//! This module provides the `ImageManager` struct which handles concurrent image downloading,
//! resizing, and rendering for anime cover images in terminal interfaces. It supports two
//! initialization modes: per-image threading and shared thread pool processing.
//!
//! # Architecture
//!
//! The ImageManager supports two threading models:
//!
//! ## Per-Image Threading (`init` + `prepare_image`)
//! - Creates a dedicated thread for each image
//! - Simple but can create many threads with large image sets
//! - Each thread handles both fetching and resizing for one image
//!
//! ## Shared Thread Pool (`init_with_threads` + `fetch_image`)  
//! - Uses two shared threads: one fetcher, one resizer
//! - More efficient for large numbers of images
//! - Fetcher thread downloads images, resizer thread processes resize requests
//!
//! # Example Usage
//!
//! ```rust
//! use std::sync::{Arc, Mutex};
//!
//! // Initialize with thread pool
//! let mut image_manager = ImageManager::new();
//! image_manager.init_with_threads(app_sender, self.get_name());
//! // where self is a screen or application context, get_name() returns something like
//! // "mainScreen"
//!
//! // Queue images for processing
//! image_manager.fetch_image(&anime);
//!
//! // In your event loop, handle the results
//! match event {
//!     Event::ImageRedraw(id, response) => {
//!         image_manager.update_image(id, response);
//!     }
//!     _ => {}
//! }
//! ```

use super::customThreadProtocol::{
    CustomResizeRequest, CustomResizeResponse, CustomThreadProtocol,
};
use crate::{
    app::Event,
    mal::{models::anime::Anime, network::fetch_image},
    screens::BackgroundUpdate,
    utils::terminalCapabilities::get_picker,
};
use ratatui::layout::Rect;
use ratatui_image::errors::Errors;
use ratatui_image::{Resize, ResizeEncodeRender, StatefulImage};

use std::sync::mpsc::{Sender, channel};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub trait HasDisplayableImage {
    fn get_displayable_image(&self) -> Option<(usize, String)>;
}

struct FetchRequest {
    id: usize,
    url: String,
}

/// Manages anime cover image downloading, resizing, and rendering for terminal applications.
///
/// The ImageManager provides two threading models for image processing:
/// - **Per-image threading**: Each image gets its own dedicated thread
/// - **Shared thread pool**: Two shared threads handle all image operations
///
/// Images are identified by anime IDs and can be rendered directly to terminal frames
/// using the ratatui library.
pub struct ImageManager {
    /// Map of anime IDs to their corresponding thread protocols
    protocols: HashMap<usize, CustomThreadProtocol>,
    /// Optional sender for communicating with the main application
    app_sx: Option<Sender<Event>>,
    /// Optional identifier for this ImageManager instance (typically screen name)
    id: Option<String>,
    /// Optional sender for custom resize requests (single-thread mode)
    image_tx: Option<Sender<CustomResizeRequest>>,
    fetcher_tx: Option<Sender<FetchRequest>>,
}

impl ImageManager {
    /// Creates a new, uninitialized ImageManager.
    ///
    /// The ImageManager must be initialized with either `init()` or `init_with_threads()`
    /// before it can process images.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let image_manager = ImageManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            protocols: HashMap::new(),
            app_sx: None,
            id: None,
            image_tx: None,
            fetcher_tx: None,
        }
    }

    pub fn clear_cache(instance: &Arc<Mutex<Self>>) {
        let mut self_lock = instance.lock().unwrap();
        self_lock.protocols.clear();
    }

    /// Initializes the ImageManager for per-image threading mode.
    ///
    /// In this mode, each call to `prepare_image()` will create a dedicated thread
    /// that handles both downloading and resizing for that specific image.
    ///
    /// # Arguments
    ///
    /// * `app_sx` - Sender channel for communicating events back to the main application
    /// * `id` - Unique identifier for this ImageManager instance (typically the screen name)
    ///
    /// # Thread Safety
    ///
    /// This method is safe to call multiple times, but will reset the ImageManager state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut image_manager = ImageManager::new();
    /// image_manager.init(app_sender, self.get_name());
    /// // where self is a screen or application context, get_name() returns something like
    /// // "mainScreen"
    ///
    /// // Now you can use prepare_image() to process individual images
    /// image_manager.prepare_image(&anime);
    /// ```
    pub fn init(instance: &Arc<Mutex<Self>>, app_sx: Sender<Event>, id: String) {
        let mut self_lock = instance.lock().unwrap();
        self_lock.app_sx = Some(app_sx);
        self_lock.id = Some(id);
    }

    /// Initializes the ImageManager with a shared thread pool for efficient bulk processing.
    ///
    /// This method creates two background threads:
    /// 1. **Fetcher Thread**: Downloads images from URLs and creates resize protocols
    /// 2. **Resizer Thread**: Processes resize requests from all images
    ///
    /// This approach is more efficient when processing many images as it limits
    /// the total number of threads to two, regardless of image count.
    ///
    /// # Arguments
    ///
    /// * `app_sx` - Sender channel for communicating events back to the main application
    /// * `id` - Unique identifier for this ImageManager instance (typically the screen name)
    ///
    /// # Thread Lifecycle
    ///
    /// The background threads will run until the ImageManager is dropped or the
    /// application terminates. They automatically handle thread cleanup when
    /// their channels are closed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut image_manager = ImageManager::new();
    /// image_manager.init_with_threads(app_sender, self.get_name());
    /// // where self is a screen or application context, get_name() returns something like
    /// // "mainScreen"
    ///
    /// // Queue multiple images efficiently
    /// for anime in anime_list {
    ///     image_manager.fetch_image(&anime);
    /// }
    /// ```
    pub fn init_with_dedicated_thread(
        instance: &Arc<Mutex<Self>>,
        app_sx: Sender<Event>,
        id: String,
    ) {
        {
            let mut self_lock = instance.lock().unwrap();
            self_lock.app_sx = Some(app_sx.clone());
            self_lock.id = Some(id.clone());
        }
        let (fetcher_tx, fetcher_rx) = channel::<FetchRequest>();
        let (image_tx, image_rx) = channel::<CustomResizeRequest>();

        {
            let mut self_lock = instance.lock().unwrap();
            self_lock.image_tx = Some(image_tx.clone());
            self_lock.fetcher_tx = Some(fetcher_tx);
        }

        let instance_clone_1 = Arc::clone(&instance);
        let instance_clone_2 = Arc::clone(&instance);
        let app_sx = app_sx.clone();
        let app_sx2 = app_sx.clone();

        std::thread::spawn(move || {
            while let Ok(req) = fetcher_rx.recv() {
                let anime_id = req.id;
                if let Ok(dyn_img) = fetch_image(req.url) {
                    let picker = get_picker();
                    let protocol = picker.new_resize_protocol(dyn_img);
                    let thread_protocol =
                        CustomThreadProtocol::new(anime_id, image_tx.clone(), Some(protocol));

                    {
                        instance_clone_1
                            .lock()
                            .unwrap()
                            .load_image(anime_id, thread_protocol);
                    }

                    let _ = app_sx2.send(Event::Rerender);
                } else {
                    eprintln!("Failed to fetch image for anime ID {}", anime_id);
                }
            }
        });

        // single thread for image resizing
        std::thread::spawn(move || {
            while let Ok(request) = image_rx.recv() {
                let anime_id = request.image_id();
                let result = request.resize_encode();

                {
                    instance_clone_2
                        .lock()
                        .unwrap()
                        .update_image(anime_id, result);
                }

                let _ = app_sx.send(Event::Rerender);
            }
        });
    }

    /// Downloads and processes an anime cover image using per-image threading.
    ///
    /// This method creates a dedicated thread for the specified anime that will:
    /// 1. Download the image from the anime's medium picture URL
    /// 2. Create a resize protocol for terminal display
    /// 3. Handle all resize requests for this image
    ///
    /// **Note**: This method should only be used when the ImageManager was initialized
    /// with `init()`. For thread pool mode, use `fetch_image()` instead.
    ///
    /// # Arguments
    ///
    /// * `anime` - The anime whose cover image should be downloaded and processed
    ///
    /// # Behavior
    ///
    /// - Returns early if the ImageManager is not initialized
    /// - Skips processing if the anime image is already loaded
    /// - Creates a background thread that terminates when resize requests stop
    ///
    /// # Thread Safety
    ///
    /// Each call creates an independent thread. The thread will automatically terminate
    /// when the image is removed from the ImageManager or the application shuts down.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Initialize with per-image threading
    /// image_manager.init(app_sender, self.get_name());
    /// // where self is a screen or application context, get_name() returns something like
    /// // "mainScreen"
    ///
    /// // Process individual images
    /// image_manager.prepare_image(&anime1);
    /// image_manager.prepare_image(&anime2);
    /// ```
    pub fn prepare_image(instance: &Arc<Mutex<Self>>, anime: &Anime) {
        {
            let mut instance = instance.lock().unwrap();

            if instance.app_sx.is_none()
                || instance.id.is_none()
                || instance.protocols.contains_key(&anime.id)
            {
                return;
            }
        }

        let anime = anime.clone();
        let instance_clone = Arc::clone(&instance);

        std::thread::spawn(move || {
            let (image_tx, image_rx) = channel::<CustomResizeRequest>();

            match fetch_image(anime.main_picture.medium) {
                Ok(dyn_img) => {
                    let picker = get_picker();
                    let protocol = picker.new_resize_protocol(dyn_img);
                    let thread_protocol =
                        CustomThreadProtocol::new(anime.id, image_tx, Some(protocol));

                    instance_clone
                        .lock()
                        .unwrap()
                        .load_image(anime.id, thread_protocol);
                }
                Err(e) => {
                    eprintln!("Failed to fetch image: {}", e);
                    return;
                }
            }

            while let Ok(request) = image_rx.recv() {
                let anime_id = request.image_id();
                let result = request.resize_encode();
                instance_clone
                    .lock()
                    .unwrap()
                    .update_image(anime_id, result);
                let app_sx = instance_clone.lock().unwrap().app_sx.clone().unwrap();
                let _ = app_sx.send(Event::Rerender);
            }
        });
    }

    /// Queues an anime cover image for download using the shared thread pool.
    ///
    /// This method sends the anime to the fetcher thread for processing. The fetcher
    /// thread will download the image and set up the resize protocol, while the
    /// resizer thread handles all resize operations.
    ///
    /// **Note**: This method should only be used when the ImageManager was initialized
    /// with `init_with_threads()`. For per-image threading, use `prepare_image()` instead.
    ///
    /// # Arguments
    ///
    /// * `anime` - The anime whose cover image should be downloaded and processed
    ///
    /// # Behavior
    ///
    /// - Returns early if the ImageManager is not initialized with thread pool mode
    /// - Skips processing if the anime image is already loaded
    /// - Queues the anime for background processing by the fetcher thread
    ///
    /// # Performance
    ///
    /// This method is non-blocking and returns immediately. The actual image download
    /// and processing happens asynchronously in the background threads.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Initialize with shared thread pool
    /// image_manager.init_with_threads(app_sender, self.get_name());
    /// // where self is a screen or application context, get_name() returns something like
    /// // "mainScreen"
    ///
    /// // Queue multiple images efficiently
    /// for anime in anime_list {
    ///     image_manager.fetch_image(&anime);
    /// }
    /// ```
    pub fn fetch_image(instance: &Arc<Mutex<Self>>, anime: &Anime) {
        {
            let mut instance = instance.lock().unwrap();
            if instance.app_sx.is_none()
                || instance.id.is_none()
                || instance.protocols.contains_key(&anime.id)
            {
                return;
            }
        }

        let instance_clone = Arc::clone(instance);
        let anime = anime.clone();
        if let Some(image_tx) = instance.lock().unwrap().image_tx.clone() {
            std::thread::spawn(move || match fetch_image(anime.main_picture.medium) {
                Ok(dyn_img) => {
                    let picker = get_picker();
                    let protocol = picker.new_resize_protocol(dyn_img);
                    let thread_protocol =
                        CustomThreadProtocol::new(anime.id, image_tx, Some(protocol));

                    instance_clone
                        .lock()
                        .unwrap()
                        .load_image(anime.id, thread_protocol);
                    let app_sx = instance_clone.lock().unwrap().app_sx.clone().unwrap();
                    let _ = app_sx.send(Event::Rerender);
                }
                Err(e) => {
                    eprintln!("Failed to fetch image: {}", e);
                }
            });
        }
    }

    pub fn fetch_image_sequential<T: HasDisplayableImage>(instance: &Arc<Mutex<Self>>, item: &T) {
        {
            let mut instance = instance.lock().unwrap();
            let (id, url) = match item.get_displayable_image() {
                Some((id, url)) => (id, url),
                None => {
                    eprintln!("Item does not have a displayable image");
                    return;
                }
            };

            if instance.app_sx.is_none()
                || instance.id.is_none()
                || instance.protocols.contains_key(&id)
            {
                return;
            }

            if let Some(sender) = instance.fetcher_tx.clone() {
                if let Err(e) = sender.send(FetchRequest { id, url }) {
                    eprintln!("Failed to send anime to fetcher thread: {}", e);
                }
            } else {
                eprintln!("Fetcher thread is not initialized");
            }
        }
    }

    /// Registers a loaded image protocol with the ImageManager.
    ///
    /// This method is typically called internally when images are successfully
    /// downloaded and processed. It stores the thread protocol so the image
    /// can be rendered and updated.
    ///
    /// # Arguments
    ///
    /// * `id` - The anime ID to associate with this protocol
    /// * `protocol` - The custom thread protocol for handling this image
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Usually called internally, but can be used for custom protocols
    /// image_manager.load_image(anime_id, custom_protocol);
    /// ```
    pub fn load_image(&mut self, id: usize, protocol: CustomThreadProtocol) {
        if self.app_sx.is_none() {
            eprintln!("App sender is not initialized");
            return;
        }
        self.protocols.insert(id, protocol);
    }

    /// Removes an image from the ImageManager and cleans up its resources.
    ///
    /// This method removes the image protocol from the internal storage, which
    /// will cause the associated background thread to terminate automatically
    /// when its channel is closed.
    ///
    /// # Arguments
    ///
    /// * `id` - The anime ID of the image to remove
    ///
    /// # Thread Cleanup
    ///
    /// Removing an image automatically signals its background thread to terminate,
    /// preventing resource leaks from unused images.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Remove an image when it's no longer needed
    /// image_manager.remove_image(anime_id);
    /// ```
    pub fn remove_image(&mut self, id: usize) {
        self.protocols.remove(&id);
    }

    /// Renders an anime cover image to the terminal frame.
    ///
    /// This method renders the image at the specified area using ratatui's
    /// StatefulImage widget. If the image is not loaded or is currently being
    /// processed, nothing will be rendered.
    ///
    /// # Arguments
    ///
    /// * `id` - The anime ID of the image to render
    /// * `frame` - The ratatui frame to render into
    /// * `area` - The rectangular area where the image should be displayed
    ///
    /// # Behavior
    ///
    /// - Returns silently if the image is not found
    /// - Automatically handles image scaling to fit the specified area
    /// - Triggers resize operations if the area size has changed
    ///
    /// # Examples
    ///
    /// ```rust
    /// // In your render loop
    /// fn render(&mut self, frame: &mut Frame) {
    ///     let image_area = Rect::new(0, 0, 20, 10);
    ///     image_manager.render_image(anime_id, frame, image_area);
    /// }
    /// ```
    pub fn render_image<T: HasDisplayableImage>(
        instance: &Arc<Mutex<Self>>,
        item: &T,
        frame: &mut ratatui::Frame,
        area: Rect,
    ) {
        let (id, _) = match item.get_displayable_image() {
            Some((id, _)) => (id, ()),
            None => {
                eprintln!("Item does not have a displayable image");
                return;
            }
        };

        if let Ok(mut self_lock) = instance.try_lock() {
            if let Some(protocol) = self_lock.protocols.get_mut(&id) {
                frame.render_stateful_widget(
                    StatefulImage::new().resize(Resize::Scale(None)),
                    area,
                    protocol,
                );
            }
        }
    }

    /// Updates an image with the results of a resize operation.
    ///
    /// This method is typically called from the main event loop when receiving
    /// `Event::ImageRedraw` events. It updates the image protocol with the
    /// completed resize operation, allowing the image to be rendered at the new size.
    ///
    /// # Arguments
    ///
    /// * `id` - The anime ID of the image to update
    /// * `response` - The result of the resize operation, either success or error
    ///
    /// # Returns
    ///
    /// * `true` if the image was successfully updated
    /// * `false` if there was an error or the image was not found
    ///
    /// # Error Handling
    ///
    /// Resize errors are logged to stderr but do not panic the application.
    /// The image will remain in its previous state if the resize fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // In your event handling code
    /// match event {
    ///     Event::ImageRedraw(id, response) => {
    ///         image_manager.update_image(id, response);
    ///     }
    ///     _ => {}
    /// }
    ///
    /// // Or called directly from an event handler
    /// fn image_redraw(&mut self, id: usize, response: Result<CustomResizeResponse, Errors>) {
    ///     self.image_manager.lock().unwrap().update_image(id, response);
    /// }
    /// ```
    pub fn update_image(
        &mut self,
        id: usize,
        response: Result<CustomResizeResponse, Errors>,
    ) -> bool {
        if let Some(protocol) = self.protocols.get_mut(&id) {
            match response {
                Ok(completed) => protocol.update_resized_protocol(completed),
                Err(e) => {
                    eprintln!("failed to update image {}: error: {}", id, e);
                    false
                }
            }
        } else {
            eprintln!("Image with ID {} not found", id);
            false
        }
    }
}
