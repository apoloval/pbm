//
// SimProc library
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(fs)]
#![feature(io)]

mod bmp;

fn main() {
    let img = match bmp::Bitmap::load("/tmp/foo2.bmp") {
        Ok(img) => img,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };
    println!("{:?}", img);
}
