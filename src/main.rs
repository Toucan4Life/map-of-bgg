use itertools::{iproduct, Itertools};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::hash::RandomState;
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
    //get output
    let output = "bgg_RatingItemDecimated.jl";
    println!("Decimating input");
    //decimate("bgg_RatingItem.jl", output);
    println!("Reading new input");
    let file = File::open(output).unwrap();
    let ratings: Vec<Rating> = serde_json::from_reader(file).unwrap();
    println!("Number of ratings : {}", ratings.len());
    //Group rating by game
    let binding = ratings
        .iter()
        .into_group_map_by(|rating| rating.bgg_id.clone());

    let binding: HashMap<&i32, HashSet<String>, RandomState> =
        HashMap::from_iter(binding.iter().filter(|test| test.1.len() > 20).map(
            |(boardgame, vec)| {
                (
                    boardgame,
                    HashSet::from_iter(vec.iter().map(|t| t.bgg_user_name.clone())),
                )
            },
        ));
    println!(
        "Number of games with more than 20 ratings : {}",
        binding.len()
    );
    //compute jaccard similarity
    let binding2 = binding.keys().tuple_combinations().collect_vec();
    println!("Number of combinations : {}", binding2.len());
    println!("Starting Jaccard");
    let jaccard_sets = binding2
        .par_iter()
        .map(|(&set1, &set2)| (set1, set2, jaccard(&binding[set1], &binding[set2])))
        .filter(|(&set1, &set2, jac)| jac.ne(&1.0));
    let binding = jaccard_sets.collect::<Vec<_>>();
    let sorted_by = binding
        .iter()
        .sorted_by(|a, b| b.2.partial_cmp(&a.2).unwrap())
        .take(100000)
        .collect_vec();

    sorted_by
        .iter()
        .take(10)
        .for_each(|chosen_one| println!("{}, {}, {}", chosen_one.0, chosen_one.1, chosen_one.2));
    
    println!("Writing File");
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("bgg_RatingJaccard.jl")
        .unwrap();

    serde_json::to_writer(file, &sorted_by).unwrap();
}

fn jaccard(set1: &HashSet<String>, set2: &HashSet<String>) -> f32 {
    (set1.intersection(&set2).count() as f32) / (set1.union(&set2).count() as f32)
}

fn decimate(input: &str, output: &str) {
    let file = File::open(input).unwrap();
    let reader = BufReader::new(file);
    let decimated: Vec<_> = reader
        .lines()
        .enumerate()
        //.par_bridge().into_par_iter()
        .filter_map(|(i, rating)| {
            let var_name = rating.unwrap();
            if i % 10 != 0 {
                return None;
            }
            match serde_json::from_str::<FullRating>(&var_name) {
                Ok(t) => {
                    if t.bgg_user_rating == None {
                        return None;
                    }
                    //println!("{}", var_name);
                    return Some(t);
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
