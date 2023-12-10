use crate::render::uniforms::Std140;

/// Create a [`wgpu::Buffer`] with an arbitrary object as contents.
pub fn with_value<T: Std140>(
    device: &wgpu::Device,
    label: Option<&str>,
    value: &T,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    let contents = value.bytes();

    use wgpu::util::DeviceExt;

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label,
        contents,
        usage,
    })
}
