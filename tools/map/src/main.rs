use shadow_company_tools::{data_dir::DataDir, map::Map};

fn main() {
    let fm = DataDir::new("C:\\Games\\shadow_company\\data");

    let mut file = match fm.open("maps\\training_final.mtf") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Could not open file ({:?})", e);
            return;
        }
    };

    let mut map = Map::default();
    map.load(&mut file).unwrap();

    map.objects.iter().for_each(|o| {
        println!("Object: {} {:?}", o.title, o.position);
        //
    });
}
