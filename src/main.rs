#![allow(non_snake_case)]

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{SystemTime};

fn main() {
    let file = File::open("gewichtsstuecke5.txt").expect("Failed to read file");
    let arr = read_weights(&file);

    let mut sums: HashMap<i64, Vec<i64>> = HashMap::new();

    let mut counters = vec![0; 13];

    let start = SystemTime::now();
    recurse(&mut sums, &arr, 0, &mut counters);
    let end = start.elapsed();

    println!("{:?}", end.unwrap());
    println!("{}", sums.len());
}

static mut ACTUAL_COUNT: u64 = 0;

fn recurse(
    sums: &mut HashMap<i64, Vec<i64>>,
    arr: &Vec<Vec<i64>>,
    n: usize,
    counters: &mut Vec<i64>,
) {
    if n == arr.len() {
        let mut sum: i64 = 0;

        for i in 0..arr.len() {
            let ints = arr.get(i).unwrap();
            sum += ints[counters[i] as usize];
        }

        if sum > 10000 || sum < -10000 {
            return;
        }

        sums.insert(sum, counters.clone());
        return;
    }

    for i in 0..arr.get(n).expect("No arr at index n").len() {
        counters[n] = i as i64;
        recurse(sums, arr, n + 1, counters);
    }
}

fn generate_weight_perms(count: i64, weight: i64) -> Vec<i64> {
    let mut weights = Vec::new();
    for i in (-count)..(count + 1) {
        weights.push(i * weight);
    }
    return weights;
}

fn read_weights(file: &File) -> Vec<Vec<i64>> {
    let start = SystemTime::now();
    let reader = BufReader::new(file);

    let mut permutations = 1;

    let mut arr: Vec<Vec<i64>> = vec![];

    for line in reader.lines().skip(1) {
        let line = line.unwrap();

        let weight: i64 = line
            .split(" ")
            .nth(0)
            .expect("Failed to read weight")
            .parse()
            .expect("Failed to parse weight");

        let count: i64 = line
            .split(" ")
            .nth(1)
            .expect("Failed to read weight-count")
            .parse()
            .expect("Failed to parse weight-count");

        permutations *= 2 * count + 1;

        arr.push(generate_weight_perms(count, weight))
    }

    println!(
        "Successfully generated weights from file.\n\t> Finished in {:?}\n\t> Estimated sums: {}",
        start.elapsed().unwrap_or_default(),
        permutations
    );

    return arr;
}

//4294967296
//4636393543
