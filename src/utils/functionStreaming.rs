pub struct StreamableRunner {
    max_batch_size: usize,

    max_of_batches: Option<usize>,
    early_stop: bool,

    // for changeing 
    new_batch_size: Option<usize>,
    new_batch_index: usize,
}

/// default batch size 20
impl StreamableRunner
{
    pub fn new() -> Self {
        StreamableRunner {
            max_batch_size: 20,

            max_of_batches: None,
            early_stop: false,

            new_batch_size: None,
            new_batch_index: 0,
        }
    }
    pub fn with_batch_size(mut self, size: usize) -> Self {
        if size > 0 {
            self.max_batch_size = size;
        }
        self
    }

    pub fn stop_early(mut self) -> Self {
        self.early_stop = true;
        self
    }

    pub fn change_batch_size_at(mut self, new_size: usize, index: usize) -> Self {
        if new_size > 0 {
            self.new_batch_index = index;
            self.new_batch_size = Some(new_size);
        }
        self
    }

    pub fn stop_at(mut self, limit: usize) -> Self {
        self.max_of_batches = Some(limit);
        self
    }

    // pub fn with_static_function<T, P>(self, static_function: P) -> StreamableRunner<T, P>
    // where
    //     P: FnMut(Vec<T>),
    // {
    //     StreamableRunner {
    //         max_batch_size: self.max_batch_size,
    //         max_of_batches: self.max_of_batches,
    //         early_stop: self.early_stop,
    //         new_batch_size: self.new_batch_size,
    //         new_batch_index: self.new_batch_index,
    //         static_function: Some(static_function),
    //         _marker: std::marker::PhantomData,
    //     }
    // }

    pub fn run<T, F>(&self, mut fetch_fn: F) -> impl Iterator<Item = Vec<T>>
    where
        F: FnMut(usize, usize) -> Option<Vec<T>>,
        T: std::fmt::Debug + Clone,
    {
        let mut offset: usize = 0;
        let mut early_stop = false;
        let mut iteration = 0;
        let mut max_batch_size= self.max_batch_size;

        std::iter::from_fn(move || {
            if early_stop {
                return None;
            }

            if let Some(limit) = self.max_of_batches{
                if limit == iteration {
                    return None;
                }
            }


            if let Some(new_size) = self.new_batch_size {
                if iteration == self.new_batch_index {
                    max_batch_size = new_size;
                }
            }

            let batch = fetch_fn(offset, max_batch_size)?;
            if batch.is_empty() {
                return None;
            }

            let batch_len = batch.len();
            offset += max_batch_size;

            if self.early_stop && batch_len < max_batch_size {
                early_stop = true;
            }

            iteration += 1;
            Some(batch)
        })
    }
}
