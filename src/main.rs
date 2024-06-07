use core::hash;
use deunicode::deunicode;
use graphviz_rust::dot_generator::graph;
use itertools::Itertools;
use quantogram::Quantogram;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
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

//10s for dec 100
//60s for dec 10
//423s for dec 1
//cargo test step1 --release
#[test]
fn step1_parse_input() {
    let output = "bgg_RatingItemDecimatedfull.jl";
    let decimated = parse("bgg_RatingItem.jl", 1);

    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output)
        .unwrap();

    serde_json::to_writer(file, &decimated).unwrap();
}
//5s for dec 100
//188s for dec 10
//678s for dec 1
//cargo test step2 --release
#[test]
fn step2_compute_jaccard() {
    let ratings_threshold = 500;
    let file = File::open("bgg_RatingItemDecimatedfull.jl").unwrap();
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
        .open("bgg_RatingJaccardfull.jl")
        .unwrap();

    serde_json::to_writer(file, &jaccard_similarities_sorted).unwrap();
}
use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use graphviz_rust::printer::{DotPrinter, PrinterContext};
//1s for dec 100
//82s for dec 10
//cargo test step3 --release
#[test]
fn step3_write_dot() {
    let file = File::open("bgg_RatingJaccardfull.jl").unwrap();
    let ratings: Vec<(i32, i32, f32)> = serde_json::from_reader(file).unwrap();
    let mut index = 0;
    let mut hashmap = HashMap::new();
    let dictionary_label: HashMap<_, _> = csv::Reader::from_path("bgg_GameItem.csv")
        .unwrap()
        .records()
        .map(|result| {
            let record = result.unwrap();
            (record[0].to_owned(), deunicode(&record[1]).replace(".", ""))
        })
        .collect();
    let mut test1 = ratings
        .iter()
        .filter(|(_, _, weigth)| weigth > &0.06) //0.01 & 0.02OK
        .map(|(a, b, w)| {
            let truea = fun_name(&mut hashmap, *a, *w, &mut index);
            let trueb = fun_name(&mut hashmap, *b, *w, &mut index);
            stmt!(edge!(node_id!(a) => node_id!(b), vec![attr!("weight",w)]))
        })
        .collect_vec();
    let mut nodes = hashmap
        .iter()
        .map(|(label, (id, weight))| {
            let name = "\"".to_owned() + dictionary_label.get(&label.to_string()).unwrap() + "\"";
            stmt!(node!(label;attr!("weight",weight),attr!("label",name)))
        })
        .collect_vec();
    nodes.append(&mut test1);
    let test = Graph::DiGraph {
        id: id!("bgg_map"),
        strict: true,
        stmts: nodes,
    };

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("bgg_RatingJaccardfull006WithName.dot")
        .unwrap();
    let ctx = &mut PrinterContext::default();
    ctx.with_semi();
    ctx.always_inline();
    file.write_all(&test.print(ctx).as_bytes());
}

fn fun_name(hashmap: &mut HashMap<i32, (i32, f32)>, a: i32, w: f32, index: &mut i32) -> i32 {
    match hashmap.get_mut(&a) {
        Some((index, weight)) => {
            *weight += w;
            *index
        }
        None => {
            hashmap.insert(a, (*index, 0.0));
            *index += 1;
            *index
        }
    }
}

use statrs::distribution::{Continuous, ContinuousCDF, FisherSnedecor};

fn jaccard_index(set1: &Vec<u32>, set2: &Vec<u32>) -> f64 {
    let intersection = set1.iter().filter(|&x| set2.contains(x)).count() as usize;
    let union = set1.len() + set2.len() - intersection;
    intersection as f64 / union as f64
}
//look for contengency table
fn jaccard_pvalue(set1: &Vec<u32>, set2: &Vec<u32>) -> f64 {
    let observed_jaccard = jaccard_index(set1, set2);
    let n1 = set1.len() as f64;
    let n2 = set2.len() as f64;
    let n3 = observed_jaccard * n1;
    let n4 = observed_jaccard * n2;

    // Compute the p-value using Fisher's exact test
    let f = FisherSnedecor::new(n1 - n3 + 1.0, n2 - n4 + 1.0).unwrap();
    let p_value = 1.0 - f.cdf(observed_jaccard);

    p_value
}
#[test]
fn test_jaccard_index() {
    let set1 = vec![1, 2, 3, 4, 5];
    let set2 = vec![4, 5, 6, 7, 8];
    println!("{}", jaccard_index(&set1, &set2));
}

#[test]
fn test_is_jaccard_significant() {
    let set1 = vec![1, 2, 3, 4, 5];
    let set2 = vec![4, 5, 6, 7, 8];
    println!("{}", jaccard_pvalue(&set1, &set2));

    let set3 = vec![1, 2, 3];
    let set4 = vec![4, 5, 6, 7, 8];
    println!("{}", jaccard_pvalue(&set3, &set4));
}

#[test]
fn test_csv() {
    let dictionary_label: HashMap<_, _> = csv::Reader::from_path("bgg_GameItem.csv")
        .unwrap()
        .records()
        .map(|result| {
            let record = result.unwrap();
            (record[0].to_owned(), record[1].to_owned())
        })
        .collect();
}

#[test]
fn find_quantiles() {
    let paths = fs::read_dir("./src/graphs").unwrap();
    let files = vec![
        fs::read_to_string("./src/graphs/0.dot").unwrap(),
        fs::read_to_string("./src/graphs/1.dot").unwrap(),
        fs::read_to_string("./src/graphs/2.dot").unwrap(),
        fs::read_to_string("./src/graphs/3.dot").unwrap(),
        fs::read_to_string("./src/graphs/4.dot").unwrap(),
        fs::read_to_string("./src/graphs/5.dot").unwrap(),
        fs::read_to_string("./src/graphs/6.dot").unwrap(),
        fs::read_to_string("./src/graphs/7.dot").unwrap(),
        fs::read_to_string("./src/graphs/8.dot").unwrap(),
    ];
    let score = files
        .iter()
        .flat_map(|string| {
            string
                .lines()
                .map(|line| fun_name1(line))
                .filter(|t| t.is_some())
                .map(|t| t.unwrap())
                .collect_vec()
        })
        .collect_vec();
    println!("{}", score.len());
    let mut q = Quantogram::new();
    score.iter().for_each(|t| q.add(*t));
    println!("10% : {}", q.quantile(0.1).unwrap());
    println!("20% : {}", q.quantile(0.2).unwrap());
    println!("30% : {}", q.quantile(0.3).unwrap());
    println!("40% : {}", q.quantile(0.4).unwrap());
    println!("50% : {}", q.quantile(0.5).unwrap());
    println!("60% : {}", q.quantile(0.6).unwrap());
    println!("70% : {}", q.quantile(0.7).unwrap());
    println!("80% : {}", q.quantile(0.8).unwrap());
    println!("90% : {}", q.quantile(0.9).unwrap());
    let i = 0;
}

#[test]
fn testouille() {
    let string ="\"Pagoda\" [\"weight\"=5.834154 \"label\"=\"Pagoda\" \"size\"=21.573 \"l\"=\"7.148,7.987\" \"id\"=154003 \"rating\"=\"6.63113\" \"complexity\"=\"1.875\"]";

    let result = fun_name1(string);

    println!("{}", result.unwrap())
}

fn fun_name1(line: &str) -> Option<f64> {
    let index = line.find("rating")?;
    match line[index + 9..index + 12].parse::<f64>() {
        Ok(value) => Some(value),
        Err(value) => {
            println!("{}", line[index + 9..index + 16].to_string());
            None
        }
    }
}
