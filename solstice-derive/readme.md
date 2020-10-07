# Shader Interface
What do I want out of a friendly Shader interface?

- Type-safe interface with uniforms.
- Temporary binding of shader (get CURRENT, set SELF, OP, set CURRENT)
- Optional memoization
    - Save uniform update when CPU-side cache is validated


```rust
trait ShaderUpdate {
    fn create_shader(&mut self) -> ShaderKey;
    fn get_shader_mut(&mut self) -> &mut Shader;
    fn use_shader(&mut self, shader: ShaderKey);
}
```