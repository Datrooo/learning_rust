pub fn quick_sort<T>(arr: &mut [T])
where T: Ord {
    if arr.len() <= 1 {
        return;
    }

    let p = partition(arr);
    let (left, right) = arr.split_at_mut(p);
    quick_sort(left);
    quick_sort(&mut right[1..]);
}

fn partition<T>(arr: &mut [T]) -> usize
where T:Ord {
    let pivot_index = arr.len() - 1;
    let mut score = 0;
    for i in 0..pivot_index {
        if arr[i] < arr[pivot_index] {
            arr.swap(i, score);
            score += 1;
        }
    }
    arr.swap(pivot_index, score);
    score
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn sorts_duplicates() {
        let mut v = [3, 1, 2, 1, 3, 0];
        quick_sort(&mut v);
        assert_eq!(v, [0, 1, 1, 2, 3, 3]);
    }
    
    #[test]
    fn empty_array() {
        let mut arr: [i32; 0] = [];
        quick_sort(&mut arr);
        assert_eq!(arr, []);
    }
    
    #[test]
    fn single_element() {
        let mut arr = [1];
        quick_sort(&mut arr);
        assert_eq!(arr, [1]);
    }
    
    #[test]
    fn already_sorted() {
        let mut arr = [1, 2, 3, 4, 5, 6, 7];
        quick_sort(&mut arr);
        assert_eq!(arr, [1, 2, 3, 4, 5, 6, 7]);
    }
    
    #[test]
    fn reverse_sorted() {
        let mut arr = [9, 8, 7, 6, 5, 4, 3, 2, 1];
        quick_sort(&mut arr);
        assert_eq!(arr, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
    
    #[test]
    fn all_same_elements() {
        let mut arr = [42, 42, 42, 42, 42];
        quick_sort(&mut arr);
        assert_eq!(arr, [42, 42, 42, 42, 42]);
    }
    
    #[test]
    fn two_elements() {
        let mut arr = [5, 3];
        quick_sort(&mut arr);
        assert_eq!(arr, [3, 5]);
    }
    
    #[test]
    fn u64_type() {
        let mut arr: [u64; 5] = [18446744073709551615, 1, 9223372036854775808, 100, 50];
        quick_sort(&mut arr);
        assert_eq!(arr, [1, 50, 100, 9223372036854775808, 18446744073709551615]);
    }
    
    #[test]
    fn chars() {
        let mut arr = ['d', 'a', 'c', 'b', 'e'];
        quick_sort(&mut arr);
        assert_eq!(arr, ['a', 'b', 'c', 'd', 'e']);
    }
    
    #[test]
    fn strings() {
        let mut arr = ["zebra", "apple", "mango", "banana"];
        quick_sort(&mut arr);
        assert_eq!(arr, ["apple", "banana", "mango", "zebra"]);
    }
}