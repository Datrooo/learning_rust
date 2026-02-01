use sorting::sorting::*;

fn main() {
    let arr = vec![64, 34, 25, 12, 22, 11, 90];
    
    println!("Исходный массив: {:?}", arr);
    
    let mut test_arr = arr.clone();
    bubble_sort(&mut test_arr);
    println!("Bubble sort:     {:?}", test_arr);
    
    let mut test_arr = arr.clone();
    selection_sort(&mut test_arr);
    println!("Selection sort:  {:?}", test_arr);
    
    let mut test_arr = arr.clone();
    quick_sort(&mut test_arr);
    println!("Quick sort:      {:?}", test_arr);
    
    let mut test_arr = arr.clone();
    heap_sort(&mut test_arr);
    println!("Heap sort:       {:?}", test_arr);
}