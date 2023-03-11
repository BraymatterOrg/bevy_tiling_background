# Unreleased

- Remove meshes parameter when calling `BackgroundImageBundle::from_image` or `CustomBackgroundImageBundle::with_material`.
  ```rust
  // Old
  BackgroundImageBundle::from_image(image, materials, meshes)
  // New
  BackgroundImageBundle::from_image(image, materials)
  ```