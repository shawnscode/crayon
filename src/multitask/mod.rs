//! #### Multi-task Subsystem
//! This is a light-weight task scheduler with automatic load balancing. Its aim to
//! make it easy to add parallelism to your sequential code. The dependecies between
//! tasks are addressed as scoped relationships.
//!
//! #### Thanks
//! The initial implementation were mainly ports from [rayon](https://github.com/nikomatsakis/rayon.git)
//! with [MIT License](https://github.com/nikomatsakis/rayon/blob/master/LICENSE-MIT).

mod latch;
mod task;
mod threads;
mod scope;

pub use self::threads::ThreadPool;

#[cfg(test)]
mod test {
    use rand::{Rng, SeedableRng, XorShiftRng};
    use super::*;

    fn quick_sort<T: PartialOrd + Send>(master: &ThreadPool, v: &mut [T]) {
        if v.len() <= 1 {
            return;
        }

        let mid = partition(v);
        let (lo, hi) = v.split_at_mut(mid);
        master.join(|| quick_sort(master, lo), || quick_sort(master, hi));
    }

    fn partition<T: PartialOrd + Send>(v: &mut [T]) -> usize {
        let pivot = v.len() - 1;
        let mut i = 0;
        for j in 0..pivot {
            if v[j] <= v[pivot] {
                v.swap(i, j);
                i += 1;
            }
        }
        v.swap(i, pivot);
        i
    }

    #[test]
    fn qsort() {
        let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
        let mut data: Vec<_> = (0..6 * 1024).map(|_| rng.next_u32()).collect();

        let master = ThreadPool::new(4);
        assert!(master.len() == 4);
        quick_sort(master.as_ref(), &mut data);

        let mut sorted_data = data.clone();
        sorted_data.sort();

        assert_eq!(data, sorted_data);
    }

}