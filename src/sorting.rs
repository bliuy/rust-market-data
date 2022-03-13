use std::{error::Error, fmt::Debug};

pub enum InterpolationMethod {
    Linear,
    Lower,
    Higher,
    Nearest,
    Midpoint,
}

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
            let y = ((Into::<f64>::into(x)) / 100.0) * length;
            let y_floor = y.floor().approx_as::<usize>()?;
            let y_ceil = y.ceil().approx_as::<usize>()?;
            let result = (vals[y_floor - 1] + vals[y_ceil - 1])
                / U::from_f64(2.0).ok_or("Should be unreachable since conversion of 2.0 into a f64 float should always work.")?;
            Ok(result)
        })
        .collect();

    result
}

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
    fn percentile_func_test() {
        let mut x: Vec<f64> = vec![20.0, 30.0, 15.0, 75.0];
        let percentiles_array = vec![75];
        let y = percentile(&percentiles_array, &mut x).unwrap();
        println!("{:#?}", y);
    }
}
