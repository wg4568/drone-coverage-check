use std::env;
use std::fs;
use std::fs::File;
use std::process::exit;

use exif::Exif;
use exif::In;
use exif::Rational;
use exif::Tag;
use exif::Value;
use plotly::Configuration;
use plotly::{
    common::Marker,
    layout::{Center, DragMode, Mapbox, MapboxStyle, Margin},
    Layout, Plot, ScatterMapbox,
};

fn exif_to_decimal(data: &(Vec<Rational>, u8)) -> f64 {
    let deg: f64 = data.0[0].num as f64 / data.0[0].denom as f64;
    let min = data.0[1].num as f64 / data.0[1].denom as f64;
    let sec = data.0[2].num as f64 / data.0[2].denom as f64;
    let sign: f64;

    if data.1 == 83 || data.1 == 87 {
        sign = -1.0;
    } else {
        sign = 1.0;
    }

    return (deg + (min / 60.0) + (sec / 3600.0)) * sign;
}

fn get_exif_gps(exif: &Exif, tag: Tag) -> (Vec<Rational>, u8) {
    let tag_ref: Tag;

    if tag == Tag::GPSLatitude {
        tag_ref = Tag::GPSLatitudeRef;
    } else {
        tag_ref = Tag::GPSLongitudeRef;
    }

    let gps_ref = match exif.get_field(tag_ref, In::PRIMARY) {
        Some(lat) => match lat.value {
            Value::Ascii(ref v) if !v.is_empty() => v[0][0],
            _ => panic!("Value broken"),
        },
        None => panic!("Value missing"),
    };

    let gps_vec = match exif.get_field(tag, In::PRIMARY) {
        Some(lat) => match lat.value {
            Value::Rational(ref v) if !v.is_empty() => v.to_owned(),
            _ => panic!("Value broken"),
        },
        None => panic!("Value missing"),
    };

    return (gps_vec, gps_ref);
}

fn get_image_coordinates(path: &str) -> (f64, f64) {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => panic!("Problem opening the file: {error:?}"),
    };

    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();

    let exif = match exifreader.read_from_container(&mut bufreader) {
        Ok(exif) => exif,
        Err(error) => panic!("Problem reading exif: {error:?}"),
    };

    let exif_lat = get_exif_gps(&exif, Tag::GPSLatitude);
    let exif_lon = get_exif_gps(&exif, Tag::GPSLongitude);

    return (exif_to_decimal(&exif_lat), exif_to_decimal(&exif_lon));
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Useage: coverage [images directory]");
        exit(1);
    }

    let paths = fs::read_dir(args[1].to_string()).unwrap();
    let mut plot = Plot::new();

    let mut avg_lat: f64 = 0.0;
    let mut avg_lon: f64 = 0.0;
    let mut tot = 0;

    for path in paths {
        let dir_entry = path.unwrap();
        let coords = get_image_coordinates(dir_entry.path().as_os_str().to_str().unwrap());

        println!(
            "{} -> {}, {}",
            dir_entry.path().display(),
            coords.0,
            coords.1
        );

        avg_lat += coords.0;
        avg_lon += coords.1;
        tot += 1;

        let trace = ScatterMapbox::new(vec![coords.0], vec![coords.1])
            .marker(Marker::new().size(10).color("#26cbde").opacity(0.8));
        plot.add_trace(trace);
    }

    let layout = Layout::new()
        .drag_mode(DragMode::Zoom)
        .show_legend(false)
        .margin(Margin::new().top(0).left(0).bottom(0).right(0))
        .mapbox(
            Mapbox::new()
                .style(MapboxStyle::OpenStreetMap)
                .center(Center::new(avg_lat / tot as f64, avg_lon / tot as f64))
                .zoom(18),
        )
        .auto_size(true);

    plot.set_layout(layout);

    let config = Configuration::new().responsive(true).fill_frame(true);

    plot.set_configuration(config);

    plot.show();
}
