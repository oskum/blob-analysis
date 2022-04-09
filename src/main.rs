use image::GrayImage;
use imageproc::{
    contours,
    contours::{BorderType, Contour},
};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use serde::{Deserialize, Serialize};

const LINE_LEN: u32 = 2;

 ///we exept that every file is image:
fn main() {

    let args: Vec<String> =  std::env::args().collect();

    if args.len() < 3 {
        panic!("too few arguments, use: <image-folder> <output-folder>");
    }


    let input_dir = Path::new(&args[1]);
    let output_dir = Path::new(&args[2]);

    for entry in std::fs::read_dir(input_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let metadata = std::fs::metadata(&path).unwrap();
       if  metadata.is_file() {
           let mut name = path.file_name().unwrap().to_os_string().into_string().unwrap();
            name = name + ".json";
           let output_file = output_dir.join(name);

           match check_image(&Path::new(&path), &output_file) {
               Ok(_) => (),
               Err(err) => println!("{:?}",err),
           };
       }
    }


}

fn check_image(input_file: &Path, output_file: &Path) -> Result<(), String> {
    if !input_file.is_file() {
        return Err(String::from("Input file does not exist"));
    }


    let display = output_file.display();

    let image: GrayImage = match image::open(&input_file) {
        Ok(img) => img.to_luma8(),
        Err(why) =>  return Err(String::from(format!("no image found at given path: {} : {}", input_file.display(), why))),
    };

    let mut file = match File::create(&output_file) {
        Err(why) => return Err(format!("couldn't create {}: {}", display, why)),
        Ok(file) => file,
    };

    let a = generate_contours(&image);
    match file.write_all(serde_json::to_string(&a).unwrap().as_bytes()) {
        Err(why) => return Err(format!("couldn't write to {}: {}", display, why)),
        Ok(_) => (),
    };
    Ok(())
}

fn generate_contours(img: &GrayImage) -> Vec<Area> {
    let mut arr: Vec<Area> = Vec::new();
    for c in contours::find_contours::<u32>(img) {
        arr.push(turn_into_area(c));
    }
    arr
}

#[derive(Serialize, Deserialize, Debug)]
struct Area {
    points: Vec<Point>,
    border: bool,
    parent: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}

fn turn_into_area(c: Contour<u32>) -> Area {
    let mut points = Vec::<Point>::new();
    for p in c.points {
        points.push(Point {
            x: p.x as i32,
            y: p.y as i32,
        });
    }
    Area {
        points: line_simplification(&points, LINE_LEN),
        parent: c.parent,
        border: match c.border_type {
            BorderType::Hole => false,
            BorderType::Outer => true,
        },
    }
}

///uses Ramer-Douglas-Peucker simplification of curve with dist threshold.
fn line_simplification(line: &Vec<Point>, dist: u32) -> Vec<Point> {
    if line.len() < 3 {
        return line.to_vec();
    }
    let (begin, end) = if line[0] == line[line.len() - 1] {
        (line[0], line[line.len() - 2])
    } else {
        (line[0], line[line.len() - 1])
    };
    let mut dist_sq: Vec<i32> = Vec::new();
    for curr in line {
        let tmp = vec2d_dist(begin, *curr)
            - i32::pow(
                vec2d_mult(vec2d_sub(end, begin), vec2d_sub(*curr, begin)),
                2,
            ) / vec2d_dist(begin, end);
        dist_sq.push(tmp);
    }
    let maxdist = dist_sq.iter().max().unwrap();

    if maxdist < &i32::pow(dist as i32, 2) {
        return Vec::from([begin, end]);
    }
    let pos = dist_sq.iter().position(|el| el == maxdist).unwrap();

    let mut v = line_simplification(&line[..pos + 2].to_vec(), dist);
    let mut v1 = line_simplification(&line[pos + 1..].to_vec(), dist);

    v.append(&mut v1);
    v
}

fn vec2d_dist(p1: Point, p2: Point) -> i32 {
    i32::pow(p1.x - p2.x, 2) + i32::pow(p1.y - p2.y, 2)
}

fn vec2d_sub(p1: Point, p2: Point) -> Point {
    Point {
        x: p1.x - p2.x,
        y: p1.y - p2.y,
    }
}

fn vec2d_mult(p1: Point, p2: Point) -> i32 {
    p1.x * p2.x + p1.y * p2.y
}
