use std::env;
use std::path::Path;


fn main() {
    let include_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("c_inc");

    cc::Build::new()
        .file("asm_src/init.S")
        .file("asm_src/memcpy.S")
        .file("asm_src/util.S")
        .file("asm_src/data.S")
        .compile("asm");
    cc::Build::new().file("third_party/malloc/dlmalloc.c").include("c_inc").warnings(false).compile("malloc");
}
