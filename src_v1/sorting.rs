use std::{collections::binary_heap::Iter, error::Error, fmt::Debug, ops::Div};

pub enum InterpolationMethod {
    Linear,
    Lower,
    Higher,
    Nearest,
    Midpoint,
}

// Heapsort implementation

pub enum HeapError {
    NoParentNode,
}

// impl std::fmt::Display for HeapError{
//     fn
// }

// impl std::error::Error for HeapError{}

pub struct Heap<T> {
    pub queue: Vec<T>,
}

impl<T> Heap<T>
where
    T: PartialOrd + Copy + From<bool> + Debug,
{
    pub fn new() -> Self {
        let mut result: Heap<T> = Heap { queue: Vec::new() };
        result.queue.push(T::from(false));
        result
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn parent_index(&self, n: usize) -> Result<usize, Box<dyn std::error::Error>> {
        if n == 1 {
            Err("No possible parent nodes".into())
        } else {
            Ok(n.wrapping_div(2))
        }
    }

    pub fn subnode_index(&self, n: usize) -> Result<usize, Box<dyn std::error::Error>> {
        if (n * 2) + 1 > self.queue.len() {
            Err("No subnodes found.".into())
        } else {
            Ok(n * 2)
        }
    }

    pub fn bubble_up(&mut self, n: usize) {
        if n <= 1 {
        } else {
            let parent_n = match self.parent_index(n) {
                Ok(i) => i,
                Err(_e) => return,
            };
            if self.queue[parent_n] > self.queue[n] {
                self.queue.swap(parent_n, n);
                self.bubble_up(parent_n);
            } else {
            }
        }
    }

    pub fn insert(&mut self, v: T) {
        self.queue.push(v);
        let new_n = self.queue.len();
        if new_n > 1 {
            self.bubble_up(new_n - 1)
        }
    }

    pub fn bubble_down(&mut self, n: usize) {
        let first_subnode = match self.subnode_index(n) {
            Ok(i) => i,
            Err(_e) => return,
        };
        let second_subnode = first_subnode + 1;
        let mut next_index: usize = n;

        // Checking against the first node
        if first_subnode < self.queue.len() && self.queue[first_subnode] < self.queue[next_index] {
            next_index = first_subnode;
        }

        // Checking against the second node
        if second_subnode < self.queue.len() && self.queue[second_subnode] < self.queue[next_index]
        {
            next_index = second_subnode;
        }

        // Swapping the minimum index with the original index
        if next_index != n {
            self.queue.swap(next_index, n);
            self.bubble_down(next_index);
        } else {
        }
    }

    pub fn extract_min(&mut self) -> Result<T, Box<dyn Error>> {
        if self.queue.len() == 1 {
            return Err("Empty queue, no elements left".into());
        };

        let n: usize = 1;
        let m: usize = self.queue.len() - 1;

        self.queue.swap(n, m);

        let result = match self.queue.pop() {
            Some(i) => i,
            None => {
                return Err(
                    "No elements left to pop! Should be unreachable due to the checks prior."
                        .into(),
                )
            }
        };

        if self.queue.len() >= 3 {
            self.bubble_down(1);
        }

        Ok(result)
    }

    pub fn min_heapsort<U>(v: U) -> Vec<T>
    where
        U: IntoIterator<Item = T> + Clone,
    {
        // Creating an empty heap object
        let mut heap: Heap<<U as IntoIterator>::Item> = Heap::new();

        // Creating the counter for allocating the memory required for the output array
        let mut input_length = 0;

        // Adding each element into the heap object
        for element in v.into_iter() {
            heap.insert(element);
            input_length += 1;
        }

        // Creating the output array object
        let mut result: Vec<T> = Vec::with_capacity(input_length);

        // Appending the individual sorted elements into the result array
        loop {
            match heap.extract_min() {
                Ok(i) => result.push(i),
                Err(e) => break,
            };
        }

        result
    }
}

// Mergesort Implementation
pub fn mergesort<T>(v: &mut Vec<T>, low: usize, high: usize) -> ()
where
    T: PartialOrd + Copy + Debug,
{
    fn merge<T>(v: &mut Vec<T>, low: usize, middle: usize, high: usize)
    where
        T: PartialOrd + Copy + Debug,
    {
        let mut i = low;
        let mut j = middle + 1;
        let mut sorted_array: Vec<T> = Vec::new();

        while (i <= middle) && (j <= high) {
            if (v[i] < v[j]) {
                sorted_array.push(v[i]);
                i += 1;
            } else {
                sorted_array.push(v[j]);
                j += 1;
            }
        }

        if (i > middle) {
            while (j <= high) {
                sorted_array.push(v[j]);
                j += 1;
            }
        }

        if (j > high) {
            while (i <= middle) {
                sorted_array.push(v[i]);
                i += 1;
            }
        }

        dbg!(low..high);
        dbg!(&sorted_array);

        v.splice(low..high + 1, sorted_array);
    }

    if low < high {
        let middle = (low + high).div(2);
        mergesort(v, low, middle);
        mergesort(v, middle + 1, high);
        merge(v, low, middle, high);
    }
}

// Quicksort implementation
pub fn quicksort<T>(v: &mut [T], low: usize, high: usize)
where
    T: PartialOrd + Debug,
{
    fn partition<T>(arr: &mut [T], low: usize, high: usize) -> usize
    where
        T: PartialOrd + Debug,
    {
        let pivot = high;
        let mut index_ptr = low;

        for i in low..high {
            if arr[i] < arr[pivot] {
                arr.swap(i, index_ptr);
                index_ptr += 1;
            }
        }
        arr.swap(pivot, index_ptr);
        index_ptr
    }
    let pivot: usize;
    if low < high {
        pivot = partition(v, low, high);
        // dbg!(&v, low, pivot, high);
        if pivot > low {
            quicksort(v, low, pivot - 1);
        }
        if pivot < high {
            quicksort(v, pivot + 1, high);
        }
    }
}

pub fn percentile<T, U>(percentiles: &[T], vals: &mut [U]) -> Result<Vec<U>, Box<dyn Error>>
where
    T: Into<f64> + Copy,
    U: PartialOrd + num_traits::Float + Copy + num_traits::FromPrimitive + Debug,
{
    use conv::*;

    // Performing the sorting of the array
    {
        let low = 0;
        let high = vals.len() - 1;
        quicksort(vals, low, high)
    }

    // Identifying the array indexes
    let length = f64::value_from(vals.len())?;
    let result = percentiles
        .iter()
        .map(|&x| -> Result<U, Box<dyn Error>> {
            let y = ((Into::<f64>::into(x)) / 100.0) * (length-1.0);
            let y_floor = y.floor().approx_as::<usize>()?;
            let y_ceil = y.ceil().approx_as::<usize>()?;
            let result = (vals[y_floor] + vals[y_ceil])
                / U::from_f64(2.0).ok_or("Should be unreachable since conversion of 2.0 into a f64 float should always work.")?;
            Ok(result)
        })
        .collect();

    result
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn quicksort_test() {
        let mut x = vec![
            86046, 44557, 25526, 25635, 9871, 77256, 30315, 44836, 69870, 88303, 41515, 35741,
            49147, 43033, 25096, 53739, 58789, 13886, 49525, 19517,
        ];
        let low = 0;
        let high = x.len() - 1;
        quicksort(&mut x, low, high);
        let correct_result = vec![
            9871, 13886, 19517, 25096, 25526, 25635, 30315, 35741, 41515, 43033, 44557, 44836,
            49147, 49525, 53739, 58789, 69870, 77256, 86046, 88303,
        ];
        assert!(correct_result == x);
    }

    #[test]
    fn heapsort_test() {
        let x = vec![
            86046, 44557, 25526, 25635, 9871, 77256, 30315, 44836, 69870, 88303, 41515, 35741,
            49147, 43033, 25096, 53739, 58789, 13886, 49525, 19517,
        ];

        let result: Vec<i32> = Heap::min_heapsort(x);

        dbg!(result);
    }

    #[test]
    fn mergesort_test() {
        let mut x = vec![
            86046, 44557, 25526, 25635, 9871, 77256, 30315, 44836, 69870, 88303, 41515, 35741,
            49147, 43033, 25096, 53739, 58789, 13886, 49525, 19517,
        ];
        let low = 0;
        let high = x.len() - 1;

        mergesort(&mut x, low, high);

        dbg!(x);
    }

    #[test]
    fn percentile_func_test() {
        let mut x: Vec<f64> = vec![20.0, 30.0, 15.0, 75.0];
        let percentiles_array = vec![75];
        let y = percentile(&percentiles_array, &mut x).unwrap();
        println!("{:#?}", y);
    }
}
