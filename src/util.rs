use bevy::image::{ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};

pub fn make_nearest(settings: &mut ImageLoaderSettings) {
    settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());
}
