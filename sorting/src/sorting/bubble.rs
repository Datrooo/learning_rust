
pub fn bubble_sort<T>( arr: &mut [T])
where T: Ord {
    let n = arr.len();
    if n < 2 {
        return;
    }
    for i in 0..n - 1 {
        let mut was_swap = false;
        for j in 0..(n - 1 - i) {
            if arr[j] > arr[j + 1] {
                arr.swap(j, j + 1);
                was_swap = true;
            }
        }
        if !was_swap {
            break;
        }
    }
}


#[cfg(test)]
mod test{
    use super::*;
    
    #[test]
    fn empty_array() {
        let mut arr: [i32; 0] = [];
        bubble_sort(&mut arr);
        assert_eq!(arr, []);
    }
    
    #[test]
    fn single_element() {
        let mut arr = [42];
        bubble_sort(&mut arr);
        assert_eq!(arr, [42]);
    }
    
    #[test]
    fn already_sorted() {
        let mut arr = [1, 2, 3, 4, 5];
        bubble_sort(&mut arr);
        assert_eq!(arr, [1, 2, 3, 4, 5]);
    }
    
    #[test]
    fn reverse_sorted() {
        let mut arr = [5, 4, 3, 2, 1];
        bubble_sort(&mut arr);
        assert_eq!(arr, [1, 2, 3, 4, 5]);
    }
    
    #[test]
    fn with_duplicates() {
        let mut arr = [3, 1, 2, 1, 3, 0];
        bubble_sort(&mut arr);
        assert_eq!(arr, [0, 1, 1, 2, 3, 3]);
    }
    
    #[test]
    fn all_same_elements() {
        let mut arr = [7, 7, 7, 7, 7];
        bubble_sort(&mut arr);
        assert_eq!(arr, [7, 7, 7, 7, 7]);
    }

    #[test]
    fn two_elements() {
        let mut arr = [2, 1];
        bubble_sort(&mut arr);
        assert_eq!(arr, [1, 2]);
    }
    
    #[test]
    fn strings() {
        let mut arr = ["zebra", "apple", "banana", "cherry"];
        bubble_sort(&mut arr);
        assert_eq!(arr, ["apple", "banana", "cherry", "zebra"]);
    }
    
    #[test]
    fn u8_type() {
        let mut arr: [u8; 5] = [200, 50, 100, 150, 25];
        bubble_sort(&mut arr);
        assert_eq!(arr, [25, 50, 100, 150, 200]);
    }
    
    #[test]
    fn i64_type() {
        let mut arr: [i64; 4] = [1000000000, -1000000000, 0, 500000000];
        bubble_sort(&mut arr);
        assert_eq!(arr, [-1000000000, 0, 500000000, 1000000000]);
    }
    
    #[test]
    fn chars() {
        let mut arr = ['z', 'a', 'x', 'b', 'c'];
        bubble_sort(&mut arr);
        assert_eq!(arr, ['a', 'b', 'c', 'x', 'z']);
    }
}

