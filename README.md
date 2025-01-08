An in progress (expect breaking changes) plugin for the Bevy game engine to support tiling backgrounds.

Use `cargo run --example tiling` to see the example.

Supports Parallax, Layered, Tiling backgrounds and makes them easy to add in bevy. 

The scrolling function is exposed as a shader import making it available for use in more specialized shaders than the provided BackgroundMaterial.

![bevy_tiling](https://user-images.githubusercontent.com/77391373/212493042-b3bd2f07-7238-42e0-ae35-edec0157eee7.gif)

### Compatible Versions

| bevy | bevy_tiling_background |
|------|------------------------|
| 0.15 | 0.12                   |
| 0.14 | 0.11                   |
| 0.9  | 0.9                    |

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
