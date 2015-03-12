## What's PBM?

PBM is the *p*ack *b*it*m*ap tool, a command line utility to process a 
BMP file to generate a packed bitmap with the following properties:

* Reduced payload. PBM format is a lightweight format. It includes almost 
no additional information but the pixel data. 

* No custom palette. Every pixel is mapped to the corresponding color
in the default palette of VGA mode 13h. 

* No well-known data format. It is not _trivial_ to read its contents. Well,
at least not as trivial as BMP, PNG, or other popular image formats. 

These properties make PBM a good file format for storing image data in 
game programming. 

# Build

`pbm` is written in Rust. 

Go to [Rust website](http://rust-lang.org/), download and install the latest 
Rust distribution. Then, from your working copy execute:

```
cargo build
```

This will download all dependencies and build the binary in `target/pbm`. 
