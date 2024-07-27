pub fn batch_items_by_cpu_count<TItem: Clone>(items: &[TItem]) -> Vec<Vec<TItem>> {
    let batch_count = num_cpus::get();
    let items_per_batch = (items.len() as f32 / batch_count as f32).ceil() as usize;

    let mut batches = Vec::new(); 

    for i in 0..batch_count {
        let start = i * items_per_batch;
        let end = ((i + 1) * items_per_batch).min(items.len());

        // Use signed isize type to avoid overflow in comparison.
        if (end as isize) - start as isize > 0 {
            batches.push(Vec::from(&items[start..end]));
        }
    }

    batches
}
