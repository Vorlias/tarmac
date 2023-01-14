use fs_err as fs;

use image::{codecs::png::PngEncoder, GenericImageView};

use std::borrow::Cow;

use crate::{
    alpha_bleed::alpha_bleed,
    api::{RobloxApiClient, ImageData, Creator},
    options::{GlobalOptions, UploadImageOptions},
};

pub fn upload_image(global: GlobalOptions, options: UploadImageOptions) {
    let image_data = fs::read(options.path).expect("couldn't read input file");

    let mut img = image::load_from_memory(&image_data).expect("couldn't load image");

    alpha_bleed(&mut img);

    let (width, height) = img.dimensions();

    let mut encoded_image: Vec<u8> = Vec::new();
    PngEncoder::new(&mut encoded_image)
        .encode(&img.to_bytes(), width, height, img.color())
        .unwrap();

    let client = RobloxApiClient::from(global);

    let upload_data = ImageData {
        name: &options.name,
        description: &options.description,
        creator: Creator {
            creatorType: global.creatorType,
            creatorId: global.creatorId,
        }
    };

    let asset_id = client
        .upload_asset(Cow::Owned(encoded_image.to_vec()), upload_data)
        .expect("Roblox API request failed");

    eprintln!("Image uploaded successfully!");
    println!("rbxassetid://{asset_id}");
}
