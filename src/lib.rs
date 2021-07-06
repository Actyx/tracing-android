mod android;
mod layer;

pub fn layer(name: &str) -> std::io::Result<layer::Layer> {
    layer::Layer::new(name)
}
