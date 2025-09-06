#![allow(unused)]
//! WARNING: these docs/comments are generated using claude: 
//!
//! Image management utilities for terminal-based anime applications.
//!
//! This module provides the `ImageManager` struct which handles concurrent image downloading,
//! resizing, and rendering for anime cover images in terminal interfaces. It uses a shared
//! thread pool approach with two background threads for efficient processing.
//!
//! # Architecture
//!
//! The ImageManager uses a shared thread pool model:
//!
//! ## Shared Thread Pool (`init_with_threads` + `query_image_for_fetching`)  
//! - Uses two shared threads: one fetcher thread and one resizer thread
//! - Efficient for large numbers of images as it limits total thread count
//! - Fetcher thread downloads images, resizer thread processes resize requests
//! - Images can be queued using `query_image_for_fetching` for any type implementing `HasDisplayableImage`
//!
//! # Example Usage
//!
//! ```rust
//! use std::sync::{Arc, Mutex};
//!
//! // Initialize with thread pool
//! let image_manager = Arc::new(Mutex::new(ImageManager::new()));
//! ImageManager::init_with_threads(&image_manager, app_sender);
//!
//! // Queue images for processing
//! ImageManager::query_image_for_fetching(&image_manager, &anime);
//!
//! // Render images in your UI
//! ImageManager::render_image(&image_manager, &anime, frame, area, true);
//! ```

use super::customThreadProtocol::{
    CustomResizeRequest, CustomResizeResponse, CustomThreadProtocol,
};
use crate::{
    app::Event, mal::{models::anime::Anime, network::fetch_image}, screens::BackgroundUpdate, send_error, utils::terminalCapabilities::get_picker
};
use ratatui::layout::Rect;
use ratatui_image::errors::Errors;
use ratatui_image::{Resize, ResizeEncodeRender, StatefulImage};

use std::sync::mpsc::{Sender, channel};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Trait for types that can provide displayable image information.
///
/// This trait allows the ImageManager to work with any type that has
/// an associated image, not just Anime objects.
pub trait HasDisplayableImage {
    /// Returns the image ID and URL for display.
    ///
    /// # Returns
    ///
    /// * `Some((id, url))` - The unique ID and image URL
    /// * `None` - If no displayable image is available
    fn get_displayable_image(&self) -> Option<(usize, String)>;
}

/// Internal request structure for the fetcher thread.
struct FetchRequest {
    id: usize,
    url: String,
}

/// Manages anime cover image downloading, resizing, and rendering for terminal applications.
///
/// The ImageManager uses a shared thread pool approach with two background threads:
/// - **Fetcher Thread**: Downloads images from URLs and creates resize protocols
/// - **Resizer Thread**: Processes resize requests from all images
///
/// Images are identified by unique IDs and can be rendered directly to terminal frames
/// using the ratatui library. The manager supports any type implementing `HasDisplayableImage`.
pub struct ImageManager {
    /// Map of image IDs to their corresponding thread protocols
    protocols: HashMap<usize, CustomThreadProtocol>,
    /// Optional sender for communicating with the main application
    app_sx: Option<Sender<Event>>,
    /// Optional sender for custom resize requests (shared resizer thread)
    image_tx: Option<Sender<CustomResizeRequest>>,
    /// Optional sender for fetch requests (shared fetcher thread)
    fetcher_tx: Option<Sender<FetchRequest>>,
}

impl ImageManager {
    /// Creates a new, uninitialized ImageManager.
    ///
    /// The ImageManager must be initialized with `init_with_threads()`
    /// before it can process images.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let image_manager = Arc::new(Mutex::new(ImageManager::new()));
    /// ```
    pub fn new() -> Self {
        Self {
            protocols: HashMap::new(),
            app_sx: None,
            image_tx: None,
            fetcher_tx: None,
        }
    }

    /// Clears all cached images and protocols from the ImageManager.
    ///
    /// This method removes all stored image protocols, which will cause
    /// associated background processing to be cleaned up. Useful for
    /// memory management when switching contexts or clearing data.
    ///
    /// # Arguments
    ///
    /// * `instance` - Arc<Mutex<Self>> reference to the ImageManager instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// ImageManager::clear_cache(&image_manager);
    /// ```
    pub fn clear_cache(instance: &Arc<Mutex<Self>>) {
        let mut self_lock = instance.lock().unwrap();
        self_lock.protocols.clear();
    }

    /// Initializes the ImageManager with basic functionality (deprecated).
    ///
    /// This method only sets up the app sender but doesn't create background threads.
    /// It's maintained for compatibility but `init_with_threads` is recommended.
    ///
    /// # Arguments
    ///
    /// * `instance` - Arc<Mutex<Self>> reference to the ImageManager instance
    /// * `app_sx` - Sender channel for communicating events back to the main application
    ///
    /// # Examples
    ///
    /// ```rust
    /// ImageManager::init(&image_manager, app_sender);
    /// ```
    pub fn init(instance: &Arc<Mutex<Self>>, app_sx: Sender<Event>) {
        let mut self_lock = instance.lock().unwrap();
        self_lock.app_sx = Some(app_sx);
    }

    /// Initializes the ImageManager with a shared thread pool for efficient bulk processing.
    ///
    /// This method creates two background threads:
    /// 1. **Fetcher Thread**: Downloads images from URLs and creates resize protocols
    /// 2. **Resizer Thread**: Processes resize requests from all images
    ///
    /// This approach is efficient when processing many images as it limits
    /// the total number of threads to two, regardless of image count.
    ///
    /// # Arguments
    ///
    /// * `instance` - Arc<Mutex<Self>> reference to the ImageManager instance
    /// * `app_sx` - Sender channel for communicating events back to the main application
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
    /// let image_manager = Arc::new(Mutex::new(ImageManager::new()));
    /// ImageManager::init_with_threads(&image_manager, app_sender);
    ///
    /// // Queue multiple images efficiently
    /// for anime in anime_list {
    ///     ImageManager::query_image_for_fetching(&image_manager, &anime);
    /// }
    /// ```
    pub fn init_with_threads(
        instance: &Arc<Mutex<Self>>,
        app_sx: Sender<Event>,
    ) {
        {
            let mut self_lock = instance.lock().unwrap();
            self_lock.app_sx = Some(app_sx.clone());
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
                let image_id = req.id;
                if let Ok(dyn_img) = fetch_image(req.url) {
                    let picker = get_picker();
                    let protocol = picker.new_resize_protocol(dyn_img);
                    let thread_protocol =
                        CustomThreadProtocol::new(image_id, image_tx.clone(), Some(protocol));

                    {
                        instance_clone_1
                            .lock()
                            .unwrap()
                            .load_image(image_id, thread_protocol);
                    }

                    let _ = app_sx2.send(Event::Rerender);
                } else {
                    send_error!("Failed to image with ID {}", image_id);
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

    /// Downloads and processes an image using per-image threading (deprecated).
    ///
    /// This method creates a dedicated thread for the specified anime that will
    /// download and process the image. This approach is less efficient than the
    /// shared thread pool and is maintained for compatibility.
    ///
    /// **Note**: This method is deprecated. Use `query_image_for_fetching()` instead
    /// after initializing with `init_with_threads()`.
    ///
    /// # Arguments
    ///
    /// * `instance` - Arc<Mutex<Self>> reference to the ImageManager instance
    /// * `anime` - The anime whose cover image should be downloaded and processed
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Deprecated - use query_image_for_fetching instead
    /// ImageManager::prepare_image(&image_manager, &anime);
    /// ```
    pub fn prepare_image(instance: &Arc<Mutex<Self>>, anime: &Anime) {
        {
            let mut instance = instance.lock().unwrap();

            if instance.app_sx.is_none()
                || instance.protocols.contains_key(&anime.id)
            {
                return;
            }
            instance.load_empy_image(anime.id);
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
                    send_error!("Failed to fetch image: {}", e);
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

    /// Queues an anime cover image for download using the shared thread pool (deprecated).
    ///
    /// This method is similar to `query_image_for_fetching` but specifically for Anime objects.
    /// It's maintained for compatibility, but `query_image_for_fetching` is more flexible.
    ///
    /// **Note**: Use `query_image_for_fetching()` instead for better type flexibility.
    ///
    /// # Arguments
    ///
    /// * `instance` - Arc<Mutex<Self>> reference to the ImageManager instance
    /// * `anime` - The anime whose cover image should be downloaded and processed
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Deprecated - use query_image_for_fetching instead
    /// ImageManager::fetch_image(&image_manager, &anime);
    /// ```
    pub fn fetch_image(instance: &Arc<Mutex<Self>>, anime: &Anime) {
        {
            let mut instance = instance.lock().unwrap();
            if instance.app_sx.is_none()
                || instance.protocols.contains_key(&anime.id)
            {
                return;
            }
            instance.load_empy_image(anime.id);
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
                    send_error!("Failed to fetch image: {}", e);
                }
            });
        }
    }

    /// Queues an image for download using the shared thread pool.
    ///
    /// This is the recommended method for requesting image downloads. It works with any
    /// type that implements `HasDisplayableImage`, making it flexible for different
    /// data types beyond just Anime objects.
    ///
    /// The method sends the item to the fetcher thread for processing. The fetcher
    /// thread will download the image and set up the resize protocol, while the
    /// resizer thread handles all resize operations.
    ///
    /// # Arguments
    ///
    /// * `instance` - Arc<Mutex<Self>> reference to the ImageManager instance
    /// * `item` - Any item implementing `HasDisplayableImage` trait
    ///
    /// # Type Parameters
    ///
    /// * `T` - Must implement `HasDisplayableImage` trait
    ///
    /// # Behavior
    ///
    /// - Returns early if the ImageManager is not initialized with thread pool mode
    /// - Skips processing if the image is already loaded or being processed
    /// - Queues the item for background processing by the fetcher thread
    /// - Creates a placeholder entry immediately to prevent duplicate requests
    ///
    /// # Performance
    ///
    /// This method is non-blocking and returns immediately. The actual image download
    /// and processing happens asynchronously in the background threads.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Initialize with shared thread pool first
    /// ImageManager::init_with_threads(&image_manager, app_sender);
    ///
    /// // Queue multiple images efficiently
    /// for anime in anime_list {
    ///     ImageManager::query_image_for_fetching(&image_manager, &anime);
    /// }
    ///
    /// // Works with any type implementing HasDisplayableImage
    /// ImageManager::query_image_for_fetching(&image_manager, &manga);
    /// ImageManager::query_image_for_fetching(&image_manager, &character);
    /// ```
    pub fn query_image_for_fetching<T: HasDisplayableImage>(instance: &Arc<Mutex<Self>>, item: &T) {
        {
            let mut instance = instance.lock().unwrap();
            let (id, url) = match item.get_displayable_image() {
                Some((id, url)) => (id, url),
                None => {
                    send_error!("Item does not have a displayable image");
                    return;
                }
            };

            if instance.app_sx.is_none()
                || instance.protocols.contains_key(&id)
            {
                return;
            }

            if let Some(sender) = instance.fetcher_tx.clone() {
                instance.load_empy_image(id);
                if let Err(e) = sender.send(FetchRequest { id, url }) {
                    send_error!("Failed to send anime to fetcher thread: {}", e);
                }
            } else {
                send_error!("Fetcher thread is not initialized (ImageManager has not been initialized properly");
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
    /// * `id` - The unique ID to associate with this protocol
    /// * `protocol` - The custom thread protocol for handling this image
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Usually called internally, but can be used for custom protocols
    /// image_manager.load_image(image_id, custom_protocol);
    /// ```
    pub fn load_image(&mut self, id: usize, protocol: CustomThreadProtocol) {
        if self.app_sx.is_none() {
            send_error!("App sender is not initialized");
            return;
        }
        self.protocols.insert(id, protocol);
    }

    /// Loads an empty placeholder protocol for an image ID.
    ///
    /// This method creates a placeholder entry in the protocols map to indicate
    /// that an image is being processed. This prevents duplicate requests for
    /// the same image while it's being downloaded.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique ID to create a placeholder for
    pub fn load_empy_image(&mut self, id: usize) {
        if self.app_sx.is_none() {
            send_error!("App sender is not initialized");
            return;
        }

        let empty_protocol = CustomThreadProtocol::empty();
        self.protocols.insert(id, empty_protocol);
    }

    /// Removes an image from the ImageManager and cleans up its resources.
    ///
    /// This method removes the image protocol from the internal storage, which
    /// will cause any associated background processing to be cleaned up automatically
    /// when channels are closed.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique ID of the image to remove
    ///
    /// # Thread Cleanup
    ///
    /// Removing an image helps with memory management and prevents resource leaks
    /// from unused images.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Remove an image when it's no longer needed
    /// image_manager.remove_image(image_id);
    /// ```
    pub fn remove_image(&mut self, id: usize) {
        self.protocols.remove(&id);
    }

    /// Renders an image to the terminal frame.
    ///
    /// This method renders the image at the specified area using ratatui's
    /// StatefulImage widget. If the image is not loaded or is currently being
    /// processed, it can optionally trigger a fetch operation.
    ///
    /// # Arguments
    ///
    /// * `instance` - Arc<Mutex<Self>> reference to the ImageManager instance
    /// * `item` - Any item implementing `HasDisplayableImage` trait
    /// * `frame` - The ratatui frame to render into
    /// * `area` - The rectangular area where the image should be displayed
    /// * `fetch_if_not_found` - Whether to automatically fetch the image if not found
    ///
    /// # Type Parameters
    ///
    /// * `T` - Must implement `HasDisplayableImage` trait
    ///
    /// # Behavior
    ///
    /// - Renders the image if it's loaded and ready
    /// - If `fetch_if_not_found` is true and the image isn't found, automatically queues it for download
    /// - Returns silently if the item doesn't have a displayable image
    /// - Automatically handles image scaling to fit the specified area
    /// - Uses a try_lock to avoid blocking if the ImageManager is busy
    ///
    /// # Examples
    ///
    /// ```rust
    /// // In your render loop
    /// fn render(&mut self, frame: &mut Frame) {
    ///     let image_area = Rect::new(0, 0, 20, 10);
    ///     
    ///     // Render with automatic fetching
    ///     ImageManager::render_image(&image_manager, &anime, frame, image_area, true);
    ///     
    ///     // Render without fetching (only show if already loaded)
    ///     ImageManager::render_image(&image_manager, &anime, frame, image_area, false);
    /// }
    /// ```
    pub fn render_image<T: HasDisplayableImage>(
        instance: &Arc<Mutex<Self>>,
        item: &T,
        frame: &mut ratatui::Frame,
        area: Rect,
        fetch_if_not_found: bool,
    ) {

        let (id, url) = match item.get_displayable_image() {
            Some((id, url)) => (id, url),
            None => {
                send_error!("Item does not have a displayable image");
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

            else if fetch_if_not_found {
                if let Some(sender) = self_lock.fetcher_tx.clone() {
                    self_lock.load_empy_image(id);
                    if let Err(e) = sender.send(FetchRequest { id, url }) {
                        send_error!("Failed to send anime to fetcher thread: {}", e);
                    }
                } else {
                    send_error!("Fetcher thread is not initialized (ImageManager has not been initialized properly");
                }
            }
        }
    }

    /// Updates an image with the results of a resize operation.
    ///
    /// This method is typically called from the main event loop when receiving
    /// image update events from the background threads. It updates the image protocol
    /// with the completed resize operation, allowing the image to be rendered at the new size.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique ID of the image to update
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
    /// // Usually called internally by the resize thread, but can be used manually
    /// let success = image_manager.update_image(image_id, resize_result);
    /// if !success {
    ///     send_error!("Failed to update image");
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
                    send_error!("failed to update image {}: error: {}", id, e);
                    false
                }
            }
        } else {
            send_error!("Image with ID {} not found", id);
            false
        }
    }
}
