macro_rules! roundup_n {
    ($a:expr, $n:expr) => {
        // origin: (((a) + ((n)-1)) & ~((n)-1))
        // https://github.com/lemire/simdjson/blob/master/include/simdjson/common_defs.h#L21
        ((($a) + (($n)-1)) & !(($n)-1))
    };
}
