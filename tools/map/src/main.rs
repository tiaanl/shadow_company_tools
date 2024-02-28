use shadow_company_tools::{fm::FileManager, map::Map};

fn main() {
    let fm = FileManager::new("C:\\Games\\shadow_company\\data");

    let mut file = match fm.open_file("maps\\training_final.mtf") {
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
