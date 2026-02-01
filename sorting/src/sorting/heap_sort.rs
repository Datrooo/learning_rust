pub fn heap_sort<T>(arr: &mut [T])
where 
    T: Ord 
{
    let size = arr.len();
    if size <= 1 {
        return;
    }

    for i in (0..(size / 2)).rev() {
        heappify(arr, i, size);
    }

    for i in (1..size).rev() {
        arr.swap(i, 0);
        heappify(arr, 0, i);
    }
}

fn heappify<T>(arr: &mut [T], i: usize, size: usize)
where 
    T: Ord 
{
    let mut largest = i;
    let left = 2 * i + 1;
    let right = 2 * i + 2;

    if left < size && arr[left] > arr[largest] {
        largest = left;
    }
    
    if right < size && arr[right] > arr[largest] {
        largest = right;
    }

    if largest != i {
        arr.swap(i, largest);
        heappify(arr, largest, size);
    }
}


#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn sorts_duplicates() {
        let mut v = [3, 1, 2, 1, 3, 0];
        heap_sort(&mut v);
        assert_eq!(v, [0, 1, 1, 2, 3, 3]);
    }
    
    #[test]
    fn empty_array() {
        let mut arr: [i32; 0] = [];
        heap_sort(&mut arr);
        assert_eq!(arr, []);
    }
    
    #[test]
    fn single_element() {
        let mut arr = [77];
        heap_sort(&mut arr);
        assert_eq!(arr, [77]);
    }
    
    #[test]
    fn already_sorted() {
        let mut arr = [1, 2, 3, 4, 5];
        heap_sort(&mut arr);
        assert_eq!(arr, [1, 2, 3, 4, 5]);
    }
    
    #[test]
    fn reverse_sorted() {
        let mut arr = [10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
        heap_sort(&mut arr);
        assert_eq!(arr, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }
    
    #[test]
    fn all_same_elements() {
        let mut arr = [3, 3, 3, 3, 3, 3];
        heap_sort(&mut arr);
        assert_eq!(arr, [3, 3, 3, 3, 3, 3]);
    }
    
    #[test]
    fn two_elements() {
        let mut arr = [8, 3];
        heap_sort(&mut arr);
        assert_eq!(arr, [3, 8]);
    }
    
    #[test]
    fn many_duplicates() {
        let mut arr = [1, 3, 1, 3, 2, 2, 1, 3];
        heap_sort(&mut arr);
        assert_eq!(arr, [1, 1, 1, 2, 2, 3, 3, 3]);
    }
    
    #[test]
    fn negative_numbers() {
        let mut arr = [-1, -5, -3, -2, -4];
        heap_sort(&mut arr);
        assert_eq!(arr, [-5, -4, -3, -2, -1]);
    }
    
    #[test]
    fn u16_type() {
        let mut arr: [u16; 5] = [65535, 1000, 30000, 5000, 10000];
        heap_sort(&mut arr);
        assert_eq!(arr, [1000, 5000, 10000, 30000, 65535]);
    }
    
    #[test]
    fn chars() {
        let mut arr = ['m', 'a', 'z', 'b', 'y'];
        heap_sort(&mut arr);
        assert_eq!(arr, ['a', 'b', 'm', 'y', 'z']);
    }
    
    #[test]
    fn strings() {
        let mut arr = ["dog", "cat", "elephant", "ant", "bear"];
        heap_sort(&mut arr);
        assert_eq!(arr, ["ant", "bear", "cat", "dog", "elephant"]);
    }
    
    #[test]
    fn i128_type() {
        let mut arr: [i128; 4] = [1000000000000000000, -1000000000000000000, 0, 500000000000000000];
        heap_sort(&mut arr);
        assert_eq!(arr, [-1000000000000000000, 0, 500000000000000000, 1000000000000000000]);
    }
}