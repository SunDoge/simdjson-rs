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
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
