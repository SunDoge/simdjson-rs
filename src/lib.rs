#[macro_use]
pub mod utils;
pub mod error;
pub mod json_parser;
pub mod parsed_json;
pub mod parsed_json_iterator;
pub mod stage1_find_marks;
pub mod stage2_build_tape;

#[cfg(test)]
mod tests {

    #[derive(Debug)]
    struct A;

    impl Drop for A {
        fn drop(&mut self) {
            println!("drop");
        }
    }

    #[test]
    fn it_works() {
        let mut a = vec![A, A];
        println!("{:?}", a);
        a = Vec::with_capacity(0);
        println!("{:?}", a);
    }
}
