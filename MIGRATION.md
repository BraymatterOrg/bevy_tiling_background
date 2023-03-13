# 0.9 to 0.10

- Remove meshes parameter when calling `BackgroundImageBundle::from_image` or `CustomBackgroundImageBundle::with_material`.
  ```rust
  // Old
  BackgroundImageBundle::from_image(image, materials, meshes)
  // New
  BackgroundImageBundle::from_image(image, materials)
- Remove the type parameter from `BackgroundMovementScale` wherever it is used in your code.

  **Before**
  ```rust
  let movement_scale = BackgroundMovementScale::<T>::default();
  let movement_scale = BackgroundMovementScale {
      scale: 0.00,
      _phantom: PhantomData::<CustomMaterial>::default(),
  };
  ```

  **After**
  ```rust
  let movement_scale = BackgroundMovementScale::default();
  let movement_scale = BackgroundMovementScale {
      scale: 0.00,
  };
  ```
- `BackgroundMovementScale::scale` has changed its value mapping. If you used the default before, it should function similar to the new default and won't require changes.
  The formula for converting to the new value should be roughly (old_value - 0.15)  + 1.0 (might not be exact). 
  The value now acts as a multiplier based on the camera speed instead of being arbitrary.
  So with a value of 1.0 it will move the same speed as the camera, making it appear stationary.
- If using a custom shader, the fragment inputs have changed and there is a new import.
  See [custombg.wgsl](https://github.com/Braymatter/bevy_tiling_background/blob/main/assets/custombg.wgsl)
  Also, where you implement [`Material2d`](https://docs.rs/bevy/0.10.0/bevy/sprite/trait.Material2d.html) for your custom material you will need to add a specialize function and vertex_shader as shown below, see the [custom](https://github.com/BraymatterOrg/bevy_tiling_background/blob/5ba24994794553a6e511fa773b3ff736dc0ef5b9/examples/custom.rs) example for more details.
  ```rust
  use bevy::core_pipeline::fullscreen_vertex_shader::FULLSCREEN_SHADER_HANDLE;

  impl Material2d for CustomMaterial {
      fn vertex_shader() -> ShaderRef {
          FULLSCREEN_SHADER_HANDLE.typed().into()
      }
      fn fragment_shader() -> ShaderRef {
          "custombg.wgsl".into()
      }

      fn specialize(
          descriptor: &mut RenderPipelineDescriptor,
          _: &MeshVertexBufferLayout,
          _: Material2dKey<Self>,
      ) -> Result<(), SpecializedMeshPipelineError> {
          descriptor.primitive = PrimitiveState::default();
          descriptor.vertex.entry_point = "fullscreen_vertex_shader".into();
          Ok(())
      }
  }
  ```