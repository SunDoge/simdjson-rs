#![feature(stdsimd)]

#[macro_use]
pub mod utils;
pub mod error;
pub mod json_parser;
pub mod parsed_json;
pub mod parsed_json_iterator;
pub mod simd_utf8_check;
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

    #[test]
    fn fantastic() {
        struct B {
            b: u32,
        }

        impl B {
            pub fn new() -> B {
                B { b: 0 }
            }

            pub fn use1(&mut self) -> u32 {
                self.b += 1;
                self.b
            }

            pub fn use2(&mut self) -> u32 {
                self.b += 2;
                self.b
            }
        }

        let mut state: Vec<fn(&mut B) -> u32> = Vec::new();
        state.push(B::use1);
        state.push(B::use2);

        let mut b = B::new();
        println!("{}", state[0](&mut b));
        println!("{}", state[1](&mut b));
    }
}
