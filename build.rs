extern crate winres;

fn main() {
    let res = winres::WindowsResource::new();

    // res.set_icon("../assets/windwaker.ico");

    // cc::Build::new()
    //     .file("src/interceptor.asm")
    //     .compile("interceptor");
    // println!("cargo:rerun-if-changed=interceptor.asm");

    res.compile().unwrap();
}
