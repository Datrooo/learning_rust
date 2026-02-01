pub fn selection_sort<T>(arr: &mut [T]) 
where T: Ord{
    let n = arr.len();
    if n < 2 {
        return;
    }
    for i in 0..(n-1) {
        let mut min_index = i;
        for j in (i + 1)..n {
            if arr[j] < arr[min_index] {
                min_index = j;
            }
        }
        if min_index != i {
            arr.swap(min_index, i);
        }
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn reverse_sorted(){
        let mut a = [6, 5, 4, 3, 2, 1];
        selection_sort(&mut a);
        assert_eq!(a, [1, 2, 3, 4, 5, 6])
    }
    
    #[test]
    fn with_duplicates(){
        let mut a = [3, 1, 2, 1, 3, 0];
        selection_sort(&mut a);
        assert_eq!(a, [0, 1, 1, 2, 3, 3])
    }
    
    #[test]
    fn empty_array() {
        let mut arr: [i32; 0] = [];
        selection_sort(&mut arr);
        assert_eq!(arr, []);
    }
    
    #[test]
    fn single_element() {
        let mut arr = [99];
        selection_sort(&mut arr);
        assert_eq!(arr, [99]);
    }
    
    #[test]
    fn already_sorted() {
        let mut arr = [1, 2, 3, 4, 5];
        selection_sort(&mut arr);
        assert_eq!(arr, [1, 2, 3, 4, 5]);
    }
    
    #[test]
    fn all_same_elements() {
        let mut arr = [5, 5, 5, 5];
        selection_sort(&mut arr);
        assert_eq!(arr, [5, 5, 5, 5]);
    }
    
    #[test]
    fn chars() {
        let mut arr = ['z', 'a', 'm', 'b', 'y'];
        selection_sort(&mut arr);
        assert_eq!(arr, ['a', 'b', 'm', 'y', 'z']);
    }
    
    #[test]
    fn u32_type() {
        let mut arr: [u32; 5] = [4000000000, 1000000000, 3000000000, 2000000000, 500000000];
        selection_sort(&mut arr);
        assert_eq!(arr, [500000000, 1000000000, 2000000000, 3000000000, 4000000000]);
    }
    
    #[test]
    fn strings() {
        let mut arr = ["rust", "python", "java", "c++"];
        selection_sort(&mut arr);
        assert_eq!(arr, ["c++", "java", "python", "rust"]);
    }
    
}