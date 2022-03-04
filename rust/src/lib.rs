use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use noise::*;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(module = "interop")]
extern {
    fn place(chunk: &JsValue, x: i32, y: i32, z: i32, block_type: u16);
}


pub struct Config {
    noise: Box<dyn NoiseFn<f64, 2>>,
    stratified_materials: Vec<Material>,
    eroded_materials: Vec<Material>,
    noise_maps: HashMap<u64, Box<dyn NoiseFn<f64, 2>>>,
    slope_cache: RefCell<HashMap<(i32, i32), f64>>,
    slope_mode: SlopeMode,
    view_dist: u8
}

impl Config {
    pub fn new(noise: Box<dyn NoiseFn<f64, 2>>, materials: Vec<Material>, slope_mode: SlopeMode, view_dist: u8) -> Self {
        let mut stratified_materials = Vec::new();
        let mut eroded_materials = Vec::new();
        let mut noise_maps = HashMap::new();

        materials.into_iter()
            .for_each(|entry| {
                noise_maps.insert(entry.id(), entry.noise());

                match entry {
                    Material::Stratified { .. } => {
                        stratified_materials.push(entry);
                    }
                    Material::Eroded { .. } => {
                        eroded_materials.push(entry);
                    }
                }
            });

        stratified_materials.sort_by(|a, b| {
            match (a, b) {
                (Material::Stratified { depth: a, .. }, Material::Stratified { depth: b, .. }) => {
                    f64::partial_cmp(a, b).unwrap()
                }
                _ => { panic!("not possible") }
            }
        });

        eroded_materials.sort_by(|a, b| {
            match (a, b) {
                (Material::Eroded { weight: a, .. }, Material::Eroded { weight: b, .. }) => {
                    u32::partial_cmp(a, b).unwrap().reverse()
                }
                _ => { panic!("not possible") }
            }
        });

        Self {
            noise,
            stratified_materials,
            eroded_materials,
            noise_maps,
            slope_cache: RefCell::default(),
            slope_mode,
            view_dist
        }
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub enum SlopeMode {
    FULL,
    VonNeumann,
    MVonNeumann,
}

impl SlopeMode {
    pub fn offsets(&self) -> Vec<(i32, i32)> {
        match self {
            SlopeMode::FULL => {
                vec![
                    (-1, -1), (-1, 0), (-1, 1),
                    ( 0, -1),          ( 0, 1),
                    ( 1, -1), ( 1, 0), ( 1, 1),
                ]
            }
            SlopeMode::VonNeumann => {
                vec![
                              (-1, 0),
                    ( 0, -1),          ( 0, 1),
                              ( 1, 0),
                ]
            }
            SlopeMode::MVonNeumann => {
                vec![
                    (-1, -1),          (-1, 1),

                    ( 1, -1),          ( 1, 1),
                ]
            }
        }
    }
}

macro_rules! material {
    (stratified; block: $block:expr, depth: $depth:expr, thickness: $thickness:expr, roughness: $roughness:expr) => {
        Material::Stratified {
            block: $block,
            depth: $depth,
            thickness: $thickness,
            roughness: $roughness,

            id: rand::random(),
        }
    };
    (eroded; block: $block:expr, top: $top:expr, weight: $weight:expr, thickness: $thickness:expr, angle: $angle:expr) => {
        Material::Eroded {
            block: $block,
            top: Some($top),
            weight: $weight,
            thickness: $thickness,
            max_slope: (($angle) as f64).to_radians().tan(),

            id: rand::random(),
        }
    };
    (eroded; block: $block:expr, weight: $weight:expr, thickness: $thickness:expr, angle: $angle:expr) => {
        Material::Eroded {
            block: $block,
            top: None,
            weight: $weight,
            thickness: $thickness,
            max_slope: ($angle as f64).to_radians().tan(),

            id: rand::random(),
        }
    };
}

#[wasm_bindgen]
pub fn init(slope_mode: SlopeMode, view_dist: u8) -> *mut Config {
    console_error_panic_hook::set_once();

    //web_sys::console::log_1(&"init".into());

    let noise: HybridMulti<OpenSimplex> = HybridMulti::new(rand::random()).set_frequency(1.0 / 128.0).set_persistence(0.7).set_octaves(8);
    let noise: Turbulence<_, OpenSimplex> = Turbulence::new(noise).set_power(10.0).set_frequency(1.0 / 16.0);
    let noise = Add::new(noise, Constant::new(1.4));
    let noise = Multiply::new(noise, Constant::new(0.5));
    let noise = Power::new(noise, Constant::new(4.0));
    let noise = Multiply::new(noise, Constant::new(60.0));
    let noise = Add::new(noise, Constant::new(40.0));

    let config = Box::new(Config::new(
        Box::new(noise),
        vec![
            material! {
                stratified;

                block: 33, // bedrock
                depth: 0.0,
                thickness: 5.0,
                roughness: 10.0
            },
            material! {
                stratified;

                block: 16093, // black stone
                depth: 4.0,
                thickness: 20.0,
                roughness: 1.0
            },
            material! {
                stratified;

                block: 20336, // smooth basalt
                depth: 20.0,
                thickness: 20.0,
                roughness: 1.0
            },
            material! {
                stratified;

                block: 18683, // deep slate
                depth: 40.0,
                thickness: 20.0,
                roughness: 1.0
            },
            material! {
                stratified;

                block: 1, // stone
                depth: 60.0,
                thickness: 20.0,
                roughness: 1.0
            },
            material! {
                eroded;

                block: 10, // dirt
                top: 9, // grass
                weight: 100,
                thickness: 5.0,
                angle: 45.0
            },
        ],
        slope_mode,
        view_dist
    ));

    Box::into_raw(config)
}

pub fn drop_ctx(ctx: *mut Config) {
    unsafe {
        Box::from_raw(ctx);
    }
}

pub struct Layer<'a> {
    material: &'a Material,
    height: f64
}

pub enum Material {
    Stratified {
        block: u16,
        depth: f64,
        thickness: f64,
        roughness: f64,

        id: u64
    },
    Eroded {
        block: u16,
        top: Option<u16>,
        weight: u32,
        thickness: f64,
        max_slope: f64,

        id: u64
    }
}

impl Material {
    pub fn block(&self) -> u16 {
        match self {
            Material::Stratified { block, .. } => {
                *block
            }
            Material::Eroded { block, .. } => {
                *block
            }
        }
    }

    pub fn roughness(&self) -> f64 {
        match self {
            Material::Stratified { roughness, .. } => {
                *roughness
            }
            Material::Eroded { .. } => {
                1.0
            }
        }
    }

    pub fn thickness(&self) -> f64 {
        match self {
            Material::Stratified { thickness, .. } => {
                *thickness
            }
            Material::Eroded { thickness, .. } => {
                *thickness
            }
        }
    }

    pub fn top(&self) -> Option<u16> {
        match self {
            Material::Eroded { top, .. } => { *top }
            Material::Stratified { .. } => { None }
        }
    }

    pub fn id(&self) -> u64 {
        match self {
            Material::Stratified { id, .. } => {
                *id
            }
            Material::Eroded { id, .. } => {
                *id
            }
        }
    }

    pub fn noise(&self) -> Box<dyn NoiseFn<f64, 2>> {
        let noise: Fbm<OpenSimplex> = Fbm::new(self.id() as u32).set_octaves(6).set_frequency(self.roughness()/128.0);
        let noise = Add::new(noise, Constant::new(1.0));
        let noise = Multiply::new(noise, Constant::new(self.thickness()));
        Box::new(noise)
    }
}

#[wasm_bindgen]
pub fn build_chunk(chunk: &JsValue, chunk_x: i32, chunk_z: i32, ptr: *const Config) {
    build_chunk_internal(&|x, y, z, block| place(chunk, x, y + 1, z, block), chunk_x, chunk_z, ptr)
}

// needed for testing
pub fn build_chunk_internal(block_consumer: &dyn Fn(i32, i32, i32, u16), chunk_x: i32, chunk_z: i32, ptr: *const Config) {
    let config = unsafe { &*ptr };
    let stratified_materials = &config.stratified_materials;
    let eroded_materials = &config.eroded_materials;
    let render_dist = (config.view_dist as i32 / 2) * 16;

    for x in 0..16 {
        for z in 0..16 {
            // setup
            let real_x = x + chunk_x * 16;
            let real_z = z + chunk_z * 16;
            let sample_coords = [real_x as f64, real_z as f64];

            if real_x >= render_dist + 15 || real_z >= render_dist + 15 ||
                real_x <= -render_dist + 1 || real_z <= -render_dist + 1 {
                continue
            }

            // base heightmap
            let total_height = config.noise.get(sample_coords);

            // generate stratified layers
            let mut layers = Vec::new();
            let mut current_height = 0.0;
            for (idx, material) in stratified_materials.iter().enumerate() {
                let mut height = if idx == stratified_materials.len() - 1 {
                    total_height - current_height
                } else {
                    config.noise_maps.get(&material.id()).expect("Unregistered Material is used").get(sample_coords)
                };

                current_height += height;
                height -= 0f64.max(current_height-total_height);

                layers.push(Layer {
                    material,
                    height
                });

                if current_height >= total_height {
                    break;
                }
            }

            // generate eroded layers
            let mut eroded_layers = Vec::new();
            let mut eroded_height = 0.0;

            for material in eroded_materials.iter() {
                let (thickness, max_slope) = match material {
                    Material::Eroded { thickness, max_slope,.. } => { (*thickness, *max_slope) }
                    _ => { panic!("not possible") }
                };

                let slope = slope(&config.noise, &mut config.slope_cache.borrow_mut(), &config.slope_mode, real_x, real_z);

                let mut height =
                    if slope <= max_slope {
                        (max_slope - slope) / max_slope * thickness
                    } else { 0.0 };

                eroded_layers.push(Layer {
                    material,
                    height
                });

                eroded_height += height;
            }

            // merge stratified layers and eroded layers
            for layer in layers.iter_mut().rev() {
                eroded_height -= layer.height;
                if eroded_height >= 0.0 {
                    layer.height = 0.0;
                } else {
                    layer.height = -eroded_height;
                    break
                }
            }

            layers.extend(eroded_layers);

            // build the chunk
            let mut last_idx = 0;
            let mut current_idx = 0;
            let mut layer_bottom = 0.0;

            for y in 0..total_height as i32 {
                let layer = loop {
                    let layer = &layers[current_idx];
                    if y as f64 + 0.5 - layer_bottom > layer.height {
                        current_idx += 1;
                        layer_bottom += layer.height;
                    } else {
                        break layer;
                    }
                };

                if last_idx != current_idx {
                    if let Some(top) = &layers[last_idx].material.top() {
                        block_consumer(x, y-1, z, *top);
                    }
                }

                block_consumer(x, y, z, layer.material.block());

                last_idx = current_idx;
            }

            if let Some(top) = &layers[last_idx].material.top() {
                block_consumer(x, total_height as i32 , z, *top);
            }
        }
    }
}

pub fn slope(noise: &Box<dyn NoiseFn<f64, 2>>, cache: &mut HashMap<(i32, i32), f64>, slope_mode: &SlopeMode, x: i32, z: i32) -> f64 {
    *cache.entry((x, z)).or_insert_with(|| {
        let x = x as f64;
        let z = z as f64;

        let center = noise.get([x, z]);
        let mut max = 0f64;

        for (dx, dz) in slope_mode.offsets() {
            let val = noise.get([x + dx as f64, z + dz as f64]);

            max = max.max((val - center).abs());
        }

        max
    })
}
