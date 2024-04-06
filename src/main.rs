use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
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
    let decimation_factor = 10;
    let ratings_threshold = 20;

    println!("Parsing input");
    let ratings = parse("bgg_RatingItem.jl", decimation_factor);
    println!("Number of ratings : {}", ratings.len());

    let binding = group_rating_by_game(ratings, ratings_threshold);
    println!(
        "Number of games with more than {} ratings : {}",
        ratings_threshold,
        binding.len()
    );

    println!("Starting Jaccard");
    let jaccard_similarities = compute_jaccard_similarities(binding);

    jaccard_similarities
        .iter()
        .filter(|(_, _, sim)| sim.ne(&0.0))
        .sorted_by(|a, b| b.2.partial_cmp(&a.2).unwrap())
        .take(10)
        .for_each(|chosen_one| println!("{}, {}, {}", chosen_one.0, chosen_one.1, chosen_one.2));
}

fn compute_jaccard_similarities(binding: HashMap<i32, HashSet<String>>) -> Vec<(i32, i32, f32)> {
    binding
        .keys()
        .tuple_combinations()
        .collect_vec()
        .par_iter()
        .map(|(&bg1, &bg2)| {
            (bg1, bg2, {
                let set1: &HashSet<String> = &binding[&bg1];
                let set2: &HashSet<String> = &binding[&bg2];
                (set1.intersection(set2).count() as f32) / (set1.union(set2).count() as f32)
            })
        })
        .collect()
}

fn group_rating_by_game(
    ratings: Vec<Rating>,
    ratings_threshold: usize,
) -> HashMap<i32, HashSet<String>, RandomState> {
    HashMap::from_iter(
        ratings
            .iter()
            .into_group_map_by(|rating| rating.bgg_id)
            .iter()
            .filter(|test| test.1.len() > ratings_threshold)
            .map(|(&boardgame, vec)| {
                (
                    boardgame,
                    HashSet::from_iter(vec.iter().map(|t| t.bgg_user_name.clone())),
                )
            }),
    )
}

fn parse(input: &str, decimation_factor: usize) -> Vec<Rating> {
    let file = File::open(input).unwrap();
    BufReader::new(file)
        .lines()
        .enumerate()
        .filter_map(|(i, rating)| {
            let var_name = rating.unwrap();
            if i % decimation_factor != 0 {
                None
            } else {
                match serde_json::from_str::<FullRating>(&var_name) {
                    Ok(t) => {
                        t.bgg_user_rating?;
                        Some(t)
                    }
                    Err(_) => {
                        println!("{}", var_name);
                        None
                    }
                }
            }
        })
        .map(|fr| Rating {
            bgg_id: fr.bgg_id,
            bgg_user_name: fr.bgg_user_name,
        })
        .collect()
}

#[test]
fn step1_parse_input() {
    let output = "bgg_RatingItemDecimated.jl";
    let decimated = parse("bgg_RatingItem.jl", 100);

    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output)
        .unwrap();

    serde_json::to_writer(file, &decimated).unwrap();
}

#[test]
fn step2_compute_jaccard() {
    let ratings_threshold = 20;
    let file = File::open("bgg_RatingItemDecimated.jl").unwrap();
    let ratings: Vec<Rating> = serde_json::from_reader(file).unwrap();

    let aggregated_ratings = group_rating_by_game(ratings, ratings_threshold);
    let jaccard_similarities = compute_jaccard_similarities(aggregated_ratings);
    let jaccard_similarities_sorted = jaccard_similarities
        .iter()
        .filter(|(_, _, sim)| sim.ne(&0.0))
        .sorted_by(|a, b| b.2.partial_cmp(&a.2).unwrap())
        .collect_vec();

    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("bgg_RatingJaccard.jl")
        .unwrap();

    serde_json::to_writer(file, &jaccard_similarities_sorted).unwrap();
}
