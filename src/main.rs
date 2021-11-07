#![allow(non_snake_case)]

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::time::SystemTime;

const MAX_GEWICHT: i64 = 10000;
const MIN_GEWICHT: i64 = 0;

static mut MAX: i64 = i64::MAX;

type SolutionSlice = (i64, Vec<i64>, Vec<i64>);

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_file = args
        .get(1)
        .expect("Please provide the path to an input file");
    let output_file = args
        .get(2)
        .expect("Please provide the path of the output file");

    let solution = solve(input_file);
    solution.write_to_file(output_file, true);
}

fn solve(input_file_path: &str) -> Solution {
    let file = File::open(input_file_path).expect("Failed to read file");

    println!("\n+ Solving for {}", input_file_path);

    let arr = read_weights(&file);

    let mut sums: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut counters = vec![0; arr.len()];
    let start = SystemTime::now();
    recurse(&mut sums, &arr, 0, &mut counters);

    println!(
        "+ Successfully ran algorithm.\n\t> Finished in {:?}\n\t> Total sums: {}",
        start.elapsed().unwrap_or_default(),
        sums.len()
    );

    unsafe {
        MAX = i64::MAX;
    }

    let start = SystemTime::now();
    let solution = Solution::create(&sums, &arr);
    println!(
        "+ Successfully created solution.\n\t> Finished in {:?}",
        start.elapsed().unwrap_or_default()
    );
    return solution;
}

fn recurse(
    sums: &mut HashMap<i64, Vec<i64>>,
    arr: &Vec<Vec<i64>>,
    n: usize,
    counters: &mut Vec<i64>,
) {
    if n == arr.len() {
        //Summe aufzählen
        let mut sum: i64 = 0;
        for i in 0..arr.len() {
            let ints = arr.get(i).unwrap();
            sum += ints[counters[i] as usize];
        }

        //Nur einfügen, wenn das maximale Gewicht und das minimale Gewicht eingehalten werden
        unsafe {
            if sum <= MAX_GEWICHT && sum >= MIN_GEWICHT {
                sums.insert(sum, counters.clone());
            } else if sum < MAX && sum > MAX_GEWICHT {
                sums.remove(&MAX);
                MAX = sum;
                sums.insert(sum, counters.clone());
            }
        }
        return;
    }
    for i in 0..arr.get(n).expect("No arr at index n").len() {
        counters[n] = i as i64;
        recurse(sums, arr, n + 1, counters);
    }
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
        "+ Successfully generated weights from file.\n\t> Finished in {:?}\n\t> Estimated sums: {}",
        start.elapsed().unwrap_or_default(),
        permutations
    );

    return arr;
}

fn generate_weight_perms(count: i64, weight: i64) -> Vec<i64> {
    let mut weights = Vec::new();
    for i in (-count)..(count + 1) {
        weights.push(i * weight);
    }
    return weights;
}

struct Solution {
    values: Vec<SolutionSlice>,
}

impl Solution {
    fn create(sums: &HashMap<i64, Vec<i64>>, arr: &Vec<Vec<i64>>) -> Self {
        let weights = {
            let mut vec = vec![];
            for local in arr {
                vec.push(local[local.len() - 1] / ((local.len() - 1) >> 1) as i64)
            }
            vec
        };

        let mut sorted: Vec<_> = sums.iter().collect();
        sorted.sort();

        let mut output = vec![];

        for entry in sorted {
            let sum = entry.0;
            let counters = entry.1;

            let mut left = vec![];
            let mut right = vec![];

            for i in 0..weights.len() {
                let weight = weights[i];
                let multiple = arr[i][counters[i] as usize];
                let abs_count = (multiple / weight).abs();

                if multiple.is_negative() {
                    for _ in 0..abs_count {
                        left.push(weight)
                    }
                } else if multiple.is_positive() {
                    for _ in 0..abs_count {
                        right.push(weight)
                    }
                }
            }

            output.push((*sum, left, right));
        }
        Self { values: output }
    }

    fn write_to_file(&self, file_name: &str, interpolate: bool) {
        let start = SystemTime::now();

        let path = std::path::Path::new(file_name);
        if let Some(parent_path) = path.parent() {
            std::fs::create_dir_all(parent_path).unwrap();
        }
        let output_file = File::create(file_name).expect("Failed to create output-file");

        let mut writer = BufWriter::new(output_file);

        if interpolate {
            for i in MIN_GEWICHT..MAX_GEWICHT + 1 {
                let nearest = self.get_closest_combination(i);
                let sum = i; //Because approximate.
                let left = nearest.1;
                let right = nearest.2;

                let interpolated = nearest.0 != sum;

                writer
                    .write(
                        format!(
                            "weight={}, was_interpolated={}\t\tleft: {:?}\n\t\t\t\t\t\t\t\t\tright: {:?}\n\n",
                            sum,interpolated, left, right
                        )
                            .as_bytes(),
                    )
                    .expect("Failed to write to output-file");
            }
        } else {
            for nearest in &self.values {
                let sum = nearest.0; //Because approximate.
                let left = &nearest.1;
                let right = &nearest.2;

                let interpolated = false;

                writer
                    .write(
                        format!(
                            "weight={}, was_interpolated={}\t\tleft: {:?}\n\t\t\t\t\t\t\t\t\tright: {:?}\n\n",
                            sum, interpolated, left, right
                        )
                            .as_bytes(),
                    )
                    .expect("Failed to write to output-file");
            }
        }
        println!(
            "+ Successfully wrote solution to {}\n\t> Finished in {:?}\n\t> interpolation: {}",
            file_name,
            start.elapsed().unwrap_or_default(),
            interpolate
        );
    }

    fn get_closest_combination(&self, weight: i64) -> SolutionSlice {
        if weight < MIN_GEWICHT {
            return self.values[0].clone();
        }

        let res = self.values.binary_search_by(|x| x.0.cmp(&(weight as i64)));

        return match res {
            Ok(index) => self.values[index].clone(),
            Err(index) => {
                if index == self.values.len()
                    || (self.values[index].0 - weight as i64).abs()
                        > (self.values[index - 1].0 - weight as i64).abs()
                {
                    self.values[index - 1].clone()
                } else {
                    self.values[index].clone()
                }
            }
        };
    }
}
