use std::{error::Error, fmt::Debug};

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
    T: PartialOrd + Copy,
{
    pub fn new() -> Self {
        Heap { queue: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn parent_index(&self, n: usize) -> Result<usize, Box<dyn std::error::Error>> {
        if n == 1 {
            Err("No possible parent nodes".into())
        } else {
            Ok(n.wrapping_div_euclid(2))
        }
    }

    pub fn child_index(&self, n: usize) -> Result<usize, Box<dyn std::error::Error>> {
        if n * 2 >= self.queue.len() - 1 {
            Err("No child nodes found.".into())
        } else {
            Ok(n * 2)
        }
    }

    pub fn bubble_up(&mut self, n: usize) {
        if n <= 1 {
             // Cannot bubble up anymore
        } else {
            let parent_n = match self.parent_index(n) {
                Ok(i) => i,
                Err(_e) => return ,
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
        let child_n = match self.child_index(n) {
            Ok(i) => i,
            Err(_e) => return ,
        };
        let child_n_alt = child_n + 1;
        // if child_n_alt + 1 > self.queue.len() {
        //     return ();
        // }s

        // Checking the first child node
        if self.queue[n] > self.queue[child_n] {
            self.queue.swap(n, child_n);
            self.bubble_down(child_n);
        } else if (child_n_alt + 1 > self.queue.len()) && (self.queue[n] > self.queue[child_n_alt])
        {
            self.queue.swap(n, child_n_alt);
            self.bubble_down(child_n_alt)
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
        let mut x = vec![
            86046, 44557, 25526, 25635, 9871, 77256, 30315, 44836, 69870, 88303, 41515, 35741,
            49147, 43033, 25096, 53739, 58789, 13886, 49525, 19517,
        ];
        let mut heap: Heap<i32> = Heap::new();
        heap.queue.push(0);
        for num in x {
            heap.insert(num);
        }

        let mut x: i32;
        for i in 0..heap.queue.len() - 1 {
            x = heap.extract_min().unwrap();
            dbg!(x);
        }
    }

    #[test]
    fn percentile_func_test() {
        let mut x: Vec<f64> = vec![20.0, 30.0, 15.0, 75.0];
        let percentiles_array = vec![75];
        let y = percentile(&percentiles_array, &mut x).unwrap();
        println!("{:#?}", y);
    }
}
