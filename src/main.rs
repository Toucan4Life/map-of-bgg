use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufReader};
#[derive(Serialize, Deserialize)]
struct FullRating {
    bgg_id: i32,
    bgg_user_name: String,
    bgg_user_owned: Option<bool>,
    bgg_user_prev_owned: Option<bool>,
    bgg_user_rating: Option<f32>,
    item_id: String,
    updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Rating {
    bgg_id: i32,
    bgg_user_name: String,
}
fn main() {
    let output = "bgg_RatingItemDecimated.jl";
    //decimate("bgg_RatingItem.jl", output);
    let file = File::open(output).unwrap();
    let ratings: Vec<Rating> = serde_json::from_reader(file).unwrap();
    println!("{}", ratings.len());
    let binding = ratings.iter().into_group_map_by(|rating| rating.bgg_user_name.clone());
    for gr in binding.values().filter(|vec| vec.len()>1){
        println!("{:?}", gr)
    }
}

fn decimate(input: &str, output: &str) {
    let file = File::open(input).unwrap();
    let reader = BufReader::new(file);
    let decimated: Vec<_> = reader
        .lines()
        .enumerate()
        .filter_map(|(i, rating)| {
            let var_name = rating.unwrap();
            if i % 1000 != 0 {
                return None;
            }
            match serde_json::from_str::<FullRating>(&var_name) {

                Ok(t) => {
                    if t.bgg_user_rating == None{return None}
                    //println!("{}", var_name);
                    return Some(t)
                }
                Err(_) => {
                    println!("{}", var_name);
                    return None;
                }
            };
        })
        .collect();

    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output)
        .unwrap();

    serde_json::to_writer(file, &decimated).unwrap();
}
