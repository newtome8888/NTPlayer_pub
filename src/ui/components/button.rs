use sdl2::{mouse::Cursor, render::Texture, surface::Surface, image::LoadSurface};

pub struct Button {}

impl Button {
    pub fn new() -> Self {
        let data = &mut [0u8];
        let sfs = Surface::from_data(data, 40, 25, 40, sdl2::pixels::PixelFormatEnum::RGBA8888).unwrap();
        let cs = Cursor::from_surface(sfs, 0, 0).unwrap();
        cs.set();

        Self {}
    }
}

pub struct ImageButton{}

impl ImageButton {
    pub fn from_file(path: &str) -> Self {
        let sfs = Surface::from_file(path).unwrap();
        let cs = Cursor::from_surface(sfs, 0, 0).unwrap();
        cs.set();

        Self {}
    }
    
}
