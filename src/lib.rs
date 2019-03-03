
#[macro_use]
pub mod utils;
pub mod parsed_json;
pub mod parsed_json_iterator;
pub mod json_parser;
pub mod error;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
