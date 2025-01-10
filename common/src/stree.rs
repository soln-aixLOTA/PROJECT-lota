#![feature(portable_simd)]
#![allow(dead_code)]

use core::arch::x86_64::{
    _mm256_movemask_epi8, _mm256_packs_epi32, _mm_prefetch, _MM_HINT_T0,
};
use std::simd::prelude::*;
use std::mem::transmute as t;

/// A single tree node of size N (usually 16).
/// Aligned to 64 bytes for optimal cache usage.
#[repr(align(64))]
#[derive(Clone, Copy)]
pub struct TreeNode<const N: usize> {
    pub data: [u32; N],
}

impl<const N: usize> TreeNode<N> {
    pub fn new(val: u32) -> Self {
        let mut data = [u32::MAX; N];
        data[0] = val;
        TreeNode { data }
    }
}

/// A static S+ tree with branching factor B+1 and N elements per node.
/// Uses "left-max" convention for storing internal node elements.
pub struct SPlusTree<const B: usize, const N: usize> {
    /// All nodes, laid out in a single contiguous array in "packed" style.
    pub tree: Vec<TreeNode<N>>,
    /// Offsets into `tree` for the start of each level.
    pub offsets: Vec<usize>,
    /// Depth of the tree (number of levels).
    pub depth: usize,
    /// The number of values (sorted).
    pub len: usize,
}

// Prefetch helper
#[inline(always)]
fn prefetch_ptr<T>(ptr: *const T) {
    unsafe {
        _mm_prefetch(ptr as *const i8, _MM_HINT_T0);
    }
}

impl<const N: usize> TreeNode<N> {
    /// Manually uses AVX2 instructions to count how many values in self.data
    /// are < q. Then returns that count. (Branchless.)
    #[inline(always)]
    pub fn find_popcnt_simd(&self, q: u32) -> usize {
        // Split into two 8-lane sets for portable_simd
        let data_lo: Simd<u32, 8> = Simd::from_slice(&self.data[0..8]);
        let data_hi: Simd<u32, 8> = Simd::from_slice(&self.data[8..16]);

        // Reinterpret as i32 to use vpcmpgtd (signed)
        let q_simd = Simd::<i32, 8>::splat(q as i32);

        unsafe {
            let mask_lo = q_simd.simd_gt(t(data_lo));
            let mask_hi = q_simd.simd_gt(t(data_hi));

            let merged = _mm256_packs_epi32(t(mask_lo), t(mask_hi));
            let bitmask = _mm256_movemask_epi8(merged);
            // Count set bits and divide by 2 (each lane contributes 2 bits)
            bitmask.count_ones() as usize / 2
        }
    }
}

impl<const B: usize, const N: usize> SPlusTree<B, N> {
    /// Build a left-max S+ tree from sorted data in `vals`.
    pub fn new_left_tree(vals: &[u32]) -> Self {
        let len = vals.len();
        if len == 0 {
            return SPlusTree {
                tree: vec![],
                offsets: vec![0],
                depth: 1,
                len: 0,
            };
        }

        // Determine depth needed
        let mut depth = 1;
        while (B + 1).pow(depth as u32 - 1) * N < len {
            depth += 1;
        }

        // Calculate total nodes needed
        let total_nodes: usize = (0..depth)
            .map(|lvl| (B + 1).pow(lvl as u32) as usize)
            .sum();

        // Build the node array
        let mut tree = vec![TreeNode { data: [u32::MAX; N] }; total_nodes];

        // Calculate level offsets
        let mut offsets = Vec::with_capacity(depth);
        {
            let mut sum = 0;
            for lvl in 0..depth {
                offsets.push(sum);
                sum += (B + 1).pow(lvl as u32) as usize;
            }
        }

        // Fill bottom level with sorted data
        let bottom_start = offsets[depth - 1];
        let bottom_count = (B + 1).pow(depth as u32 - 1) as usize;

        let mut idx = 0;
        for node_i in 0..bottom_count {
            let node_offset = bottom_start + node_i;
            if node_offset < tree.len() {
                let node_data = &mut tree[node_offset].data;
                for j in 0..N {
                    if idx < len {
                        node_data[j] = vals[idx];
                        idx += 1;
                    }
                }
            }
        }

        // Fill internal levels bottom-up
        for lvl in (0..(depth - 1)).rev() {
            let start = offsets[lvl];
            let count = (B + 1).pow(lvl as u32) as usize;
            let child_start = offsets[lvl + 1];
            let child_count = (B + 1).pow((lvl + 1) as u32) as usize;
            let stride = child_count / count;

            for node_i in 0..count {
                let node_offset = start + node_i;
                let child_base = child_start + node_i * stride;

                if node_offset >= tree.len() {
                    continue;
                }

                // Collect child values first
                let mut child_values = [u32::MAX; B];
                let mut valid_children = 0;

                for b_i in 0..B {
                    let child_i = child_base + b_i;
                    if child_i < tree.len() {
                        child_values[b_i] = tree[child_i].data[N - 1];
                        valid_children = b_i + 1;
                    }
                }

                let last_child = child_base + B;
                let last_val = if last_child < tree.len() {
                    tree[last_child].data[N - 1]
                } else {
                    u32::MAX
                };

                // Now update the parent node
                let parent = &mut tree[node_offset].data;
                for b_i in 0..valid_children {
                    parent[b_i] = child_values[b_i];
                }
                if valid_children > 0 {
                    parent[B - 1] = last_val;
                }
            }
        }

        SPlusTree {
            tree,
            offsets,
            depth,
            len,
        }
    }

    /// Query a single value to find the smallest value >= q.
    #[inline]
    pub fn query_one(&self, q: u32) -> u32 {
        let mut node_idx = 0;
        for lvl in 0..(self.depth - 1) {
            let node = unsafe { self.tree.as_ptr().add(self.offsets[lvl] + node_idx) };
            let jump_to = unsafe { (*node).find_popcnt_simd(q) };
            node_idx = node_idx * (B + 1) + jump_to;
            let next = self.offsets[lvl + 1] + node_idx;
            prefetch_ptr(unsafe { self.tree.as_ptr().add(next) });
        }

        let leaf_node = &self.tree[self.offsets[self.depth - 1] + node_idx];
        let off_in_leaf = leaf_node.find_popcnt_simd(q).min(N - 1);
        let val = leaf_node.data[off_in_leaf];

        if val < q {
            if off_in_leaf + 1 < N {
                leaf_node.data[off_in_leaf + 1]
            } else {
                u32::MAX
            }
        } else {
            val
        }
    }

    /// Query a slice of values one at a time.
    pub fn query_slice(&self, queries: &[u32]) -> Vec<u32> {
        queries.iter().map(|&q| self.query_one(q)).collect()
    }

    /// A fully batched query version that processes queries in blocks of P.
    pub fn query_batch<const P: usize>(&self, queries: &[u32; P]) -> [u32; P] {
        let mut idxes = [0_usize; P];
        prefetch_ptr(self.tree.as_ptr());

        for lvl in 0..(self.depth - 1) {
            let base = self.offsets[lvl];
            let next_base = self.offsets[lvl + 1];

            for i in 0..P {
                let node_ptr = unsafe { self.tree.as_ptr().add(base + idxes[i]) };
                let jump_to = unsafe { (*node_ptr).find_popcnt_simd(queries[i]) };
                idxes[i] = idxes[i] * (B + 1) + jump_to;

                let prefetch_idx = next_base + idxes[i];
                prefetch_ptr(unsafe { self.tree.as_ptr().add(prefetch_idx) });
            }
        }

        let leaf_base = self.offsets[self.depth - 1];
        let mut results = [0_u32; P];
        for i in 0..P {
            let leaf_node = &self.tree[leaf_base + idxes[i]];
            let off_in_leaf = leaf_node.find_popcnt_simd(queries[i]).min(N - 1);
            let val = leaf_node.data[off_in_leaf];

            if val < queries[i] {
                if off_in_leaf + 1 < N {
                    results[i] = leaf_node.data[off_in_leaf + 1];
                } else {
                    results[i] = u32::MAX;
                }
            } else {
                results[i] = val;
            }
        }

        results
    }

    fn build_left_tree(tree: &mut [TreeNode<N>], node_offset: usize, child_base: usize) {
        let (left_tree, right_tree) = tree.split_at_mut(node_offset + 1);
        let node_data = &mut left_tree[node_offset].data;
        
        for b_i in 0..B {
            node_data[b_i] = right_tree[child_base + b_i - node_offset - 1].data[N - 1];
        }
        node_data[B - 1] = right_tree[child_base + B - node_offset - 1].data[N - 1];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use std::time::Instant;

    #[test]
    fn test_basic_functionality() {
        let mut rng = rand::thread_rng();
        let vals: Vec<u32> = (0..512).map(|_| rng.gen::<u32>() >> 1).collect();
        let mut sorted_vals = vals.clone();
        sorted_vals.sort_unstable();

        const B: usize = 16;
        const N: usize = 16;
        let stree = SPlusTree::<B, N>::new_left_tree(&sorted_vals);

        // Single query
        let q = 123456789u32 >> 1;
        let ans = stree.query_one(q);
        assert!(ans >= q);

        // Batched query
        let queries = [111u32, 2222, 50000, 999_999, 1_000_000, 555, 777, 999];
        let results = stree.query_batch(&queries);
        for (q, r) in queries.iter().zip(results.iter()) {
            assert!(*r >= *q);
        }
    }

    #[test]
    fn test_edge_cases() {
        const B: usize = 16;
        const N: usize = 16;

        // Test empty array
        let empty: Vec<u32> = vec![];
        let stree = SPlusTree::<B, N>::new_left_tree(&empty);
        assert_eq!(stree.query_one(123), u32::MAX);

        // Test single element
        let single = vec![42];
        let stree = SPlusTree::<B, N>::new_left_tree(&single);
        assert_eq!(stree.query_one(41), 42);
        assert_eq!(stree.query_one(42), 42);
        assert_eq!(stree.query_one(43), u32::MAX);

        // Test array with duplicates
        let duplicates = vec![1, 1, 1, 2, 2, 3, 3, 3];
        let stree = SPlusTree::<B, N>::new_left_tree(&duplicates);
        assert_eq!(stree.query_one(1), 1);
        assert_eq!(stree.query_one(2), 2);
        assert_eq!(stree.query_one(3), 3);
        assert_eq!(stree.query_one(4), u32::MAX);

        // Test exact node size
        let exact = (0..N as u32).collect::<Vec<_>>();
        let stree = SPlusTree::<B, N>::new_left_tree(&exact);
        for i in 0..N as u32 {
            assert_eq!(stree.query_one(i), i);
        }
        assert_eq!(stree.query_one(N as u32), u32::MAX);
    }

    #[test]
    fn test_batch_vs_sequential() {
        const BATCH_SIZE: usize = 32;
        let mut rng = rand::thread_rng();
        let mut vals: Vec<u32> = (0..1000).map(|_| rng.gen()).collect();
        vals.sort_unstable();
        
        let stree = SPlusTree::<16, 16>::new_left_tree(&vals);
        let mut queries: Vec<u32> = (0..BATCH_SIZE).map(|_| rng.gen()).collect();
        queries.sort_unstable();
        
        // Time sequential queries
        let start = Instant::now();
        let mut sequential_results = Vec::with_capacity(BATCH_SIZE);
        for &q in &queries {
            sequential_results.push(stree.query_one(q));
        }
        let _sequential_time = start.elapsed();
        
        // Time batched queries
        let start = Instant::now();
        let query_array = queries.try_into().unwrap();
        let batch_results = stree.query_batch::<BATCH_SIZE>(&query_array);
        let _batch_time = start.elapsed();
        
        assert_eq!(sequential_results, batch_results.to_vec());
    }

    #[test]
    fn test_large_range() {
        let mut vals = vec![
            0,
            u32::MAX / 4,
            u32::MAX / 2,
            3 * (u32::MAX / 4),
            u32::MAX,
        ];
        vals.sort_unstable();
        
        let stree = SPlusTree::<16, 16>::new_left_tree(&vals);
        
        // Test queries at various points in the range
        let queries = vec![
            0,
            u32::MAX / 4,
            u32::MAX / 2,
            3 * (u32::MAX / 4),
            u32::MAX,
        ];
        
        for &q in &queries {
            let result = stree.query_one(q);
            assert!(result >= q);
            if let Some(&next) = vals.iter().find(|&&x| x >= q) {
                assert_eq!(result, next);
            } else {
                assert_eq!(result, u32::MAX);
            }
        }
    }

    #[test]
    fn test_node_boundaries() {
        const B: usize = 16;
        const N: usize = 16;

        // Test arrays that are just under/over node boundaries
        let sizes = [N - 1, N, N + 1, 2 * N - 1, 2 * N, 2 * N + 1];

        for &size in &sizes {
            let vals: Vec<u32> = (0..size as u32).collect();
            let stree = SPlusTree::<B, N>::new_left_tree(&vals);

            // Test every possible query in this range
            for q in 0..size as u32 + 2 {
                let result = stree.query_one(q);
                if q >= size as u32 {
                    assert_eq!(result, u32::MAX);
                } else {
                    assert_eq!(result, q);
                }
            }
        }
    }

    #[test]
    fn test_batch_sizes() {
        const B: usize = 16;
        const N: usize = 16;

        // Create a medium-sized sorted array
        let mut rng = rand::thread_rng();
        let size = 100_000;
        let mut vals: Vec<u32> = (0..size).map(|_| rng.gen::<u32>() >> 1).collect();
        vals.sort_unstable();

        let stree = SPlusTree::<B, N>::new_left_tree(&vals);

        // Test different batch sizes
        let mut test_batch = |batch_size| {
            let queries: Vec<u32> = (0..batch_size).map(|_| rng.gen::<u32>() >> 1).collect();
            let sequential_results: Vec<u32> = queries.iter().map(|&q| stree.query_one(q)).collect();
            
            match batch_size {
                1 => {
                    let arr: [u32; 1] = queries.try_into().unwrap();
                    let batch_results = stree.query_batch(&arr);
                    assert_eq!(sequential_results, batch_results.to_vec());
                }
                2 => {
                    let arr: [u32; 2] = queries.try_into().unwrap();
                    let batch_results = stree.query_batch(&arr);
                    assert_eq!(sequential_results, batch_results.to_vec());
                }
                4 => {
                    let arr: [u32; 4] = queries.try_into().unwrap();
                    let batch_results = stree.query_batch(&arr);
                    assert_eq!(sequential_results, batch_results.to_vec());
                }
                8 => {
                    let arr: [u32; 8] = queries.try_into().unwrap();
                    let batch_results = stree.query_batch(&arr);
                    assert_eq!(sequential_results, batch_results.to_vec());
                }
                16 => {
                    let arr: [u32; 16] = queries.try_into().unwrap();
                    let batch_results = stree.query_batch(&arr);
                    assert_eq!(sequential_results, batch_results.to_vec());
                }
                32 => {
                    let arr: [u32; 32] = queries.try_into().unwrap();
                    let batch_results = stree.query_batch(&arr);
                    assert_eq!(sequential_results, batch_results.to_vec());
                }
                _ => panic!("Unsupported batch size"),
            }
        };

        for &size in &[1, 2, 4, 8, 16, 32] {
            test_batch(size);
        }
    }

    #[test]
    fn test_cache_effects() {
        const BATCH_SIZE: usize = 32;
        let mut rng = rand::thread_rng();
        let mut vals: Vec<u32> = (0..1000).map(|_| rng.gen()).collect();
        vals.sort_unstable();
        
        let stree = SPlusTree::<16, 16>::new_left_tree(&vals);
        
        // Test clustered queries
        let start = Instant::now();
        let mut clustered_queries = Vec::with_capacity(BATCH_SIZE);
        let base = rng.gen::<u32>();
        for i in 0..BATCH_SIZE {
            clustered_queries.push(base.wrapping_add(i as u32));
        }
        let query_array = clustered_queries.try_into().unwrap();
        let _clustered_results = stree.query_batch::<BATCH_SIZE>(&query_array);
        let _clustered_time = start.elapsed();
        
        // Test scattered queries
        let start = Instant::now();
        let scattered_queries: Vec<u32> = (0..BATCH_SIZE).map(|_| rng.gen()).collect();
        let query_array = scattered_queries.try_into().unwrap();
        let _scattered_results = stree.query_batch::<BATCH_SIZE>(&query_array);
        let _scattered_time = start.elapsed();
    }

    #[test]
    fn test_prefetch_effectiveness() {
        const B: usize = 16;
        const N: usize = 16;
        const BATCH_SIZE: usize = 16;
        const ITERATIONS: usize = 100;

        let mut rng = rand::thread_rng();
        let size = 1_000_000; // Large enough to exceed cache
        let mut vals: Vec<u32> = (0..size).map(|_| rng.gen::<u32>() >> 1).collect();
        vals.sort_unstable();

        let stree = SPlusTree::<B, N>::new_left_tree(&vals);

        // Run many iterations to get stable timing
        let mut total_batch_time = std::time::Duration::ZERO;
        let mut total_sequential_time = std::time::Duration::ZERO;

        for _ in 0..ITERATIONS {
            let queries: Vec<u32> = (0..BATCH_SIZE).map(|_| rng.gen::<u32>() >> 1).collect();
            let query_array: [u32; BATCH_SIZE] = queries.clone().try_into().unwrap();

            // Time sequential
            let start = Instant::now();
            let sequential_results: Vec<u32> =
                queries.iter().map(|&q| stree.query_one(q)).collect();
            total_sequential_time += start.elapsed();

            // Time batched
            let start = Instant::now();
            let batch_results = stree.query_batch(&query_array);
            total_batch_time += start.elapsed();

            // Verify results
            for (seq, batch) in sequential_results.iter().zip(batch_results.iter()) {
                assert_eq!(seq, batch);
            }
        }

        // Optional timing output (commented out)
        // let avg_sequential = total_sequential_time.as_nanos() as f64 / ITERATIONS as f64;
        // let avg_batch = total_batch_time.as_nanos() as f64 / ITERATIONS as f64;
        // println!("Avg Sequential: {:.0}ns", avg_sequential);
        // println!("Avg Batch: {:.0}ns", avg_batch);
        // println!("Speedup: {:.2}x", avg_sequential / avg_batch);
    }
}

