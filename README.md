An in progress (expect breaking changes) plugin for the Bevy game engine to support tiling backgrounds.

Use `cargo run --example tiling` to see the example.

Supports Parallax, Layered, Tiling backgrounds and makes them easy to add in bevy. 

The scrolling function is exposed as a shader import making it available for use in more specialized shaders than the provided BackgroundMaterial.

![bevy_tiling](https://user-images.githubusercontent.com/77391373/212493042-b3bd2f07-7238-42e0-ae35-edec0157eee7.gif)

### Compatible Versions

| bevy | bevy_tiling_background |
|------|------------------------|
| 0.9  | 0.9                    |

### License
Copyright 2023 BrayMatter LLC

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
