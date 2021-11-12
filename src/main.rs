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

/// The heart of the program - the algorithm for generating the sums and
/// their combinations.<p>
///
/// # How it works (all of it)
/// The following describes how the algorithm works.
/// A classic approach would mean calculating the sum of every permutation
/// of the given weights.
///
/// At first, this sounds doable, however lets take a look at the amount
/// of permutations possible.
///
/// Lets say, for a start, we can only either include or exclude the weight
/// from the sum (We'll get to the more accurate part later). This means, we
/// have 2 states per weight in a sum.
///
/// The amount of possible permutations can be calculated by 2^n where n is
/// the amount of weights in this case. This is pretty bad new for us, because
/// it means, that we have an exponential increase in computation time. There
/// are 23 total weights in `gewichtsstuecke5.txt`. This means, we have to
/// calculate a maximum of 2^23 permutations which is about 8 million. This
/// is still a reasonable amount we can compute.
///
/// However, it gets worse. Consider the actual problem - we have 3 states
/// the weight can be in. Either it can be excluded from the sum, it can influence
/// the sum with its positive weight, or it can influence the sum with its
/// negative weight. This describes placing the weight on the left or right
/// side of the scale, or excluding it altogether.
///
/// Now, instead of 2^n, we have 3^n. It doesnt look that bad at first glance.
/// However, consider this:
/// * 2^23 ≈ 8 million vs. 3^23 ≈ 94 billion
///
/// This is way out of the range we can compute in a reasonable time. It could take
/// hours to calculate all permutations.
///
/// This means, we have to find another way to calculate sums. What if we were
/// able to take advantage of the fact, that some of the weights are the same,
/// ie. have an equal weight?
///
/// With the help of an algorithm that answers this question, we are able to
/// reduce the amount of total permutations we have to compute to
/// $$\prod_{i=0}^{n}({2\cdot count(W_i)+1})$$
/// Where $n$ is the amount of distinct weights, $W_i$ is the current weight,
/// and $count()$ is a function that returns the amount of weights with the same weight.
///
/// The algorithm works by first generating all the possible multiples that can
/// be generated with the the number of weights for each distinct
/// weight.
///
/// You will get something like this:
/// ```
/// (5 3) -> | -15  | -10  | -5   |   0  |   5  |  10  |  15  |
/// (3 1) -> | -3   |  0   |  3   |      |      |      |      |
/// (4 2) -> | -8   | -4   |  0   |   4  |   8  |      |      |
/// ```
///
/// Now, the idea should slowly begin to form. Remember: we want to find
/// all the sums, and save the combination of weights for each of these
/// sums. Let's say, we only have a fixed amount of distinct weights, as
/// this makes it easier to fundamentally grasp the solution.
///
/// Imagine yourself, drawing a line (in the upper table) from top to bottom
/// through the values in the table, while adding them up. These paths represent
/// our sums ie. the combinations of the weights.
///
/// How many paths can we draw in total? The answer is quite simple:
/// Simply multiply the amounts of multiples in each row with one another.
/// For the above table, this would give a total of `7*3*5 = 105`. So, in
/// total, we are left with 105 permutations we have to iterate through to find
/// every sum.
///
/// Lets compare this value to the old algorithm with 3^n permutations.
/// In total, we have `3+1+2 = 6` weights. `3^6 = 729`, which is much
/// more than 105 permutations with our new algorithm. This is because
/// with the 3^n algorithm, we were considering each weight on it's own.
/// This is inefficient, as it leads to more permutations which have the
/// same sum being computed.
///
/// With the new algorithm, we are able to quickly iterate over each permutation,
/// and find each sum we can create with a combination of the given weights.
///
/// Let's actually look at the implementation in code:
/// How do we create the permutations? If we had a fixed amount of distinct weights,
/// we would be able to iterate over each weight, with `for-loops` like so:
/// <p>
///
/// ```rust
/// for i in 0..arr[0].len(){
///     for j in 0..arr[1].len() {
///         for k in 0..arr[2].len() {
///             let combination = vec![arr[0][i],arr[1][j],arr[2][k]];
///             let sum = combination[0] + combination[1] + combination[2];
///         }
///     }
/// }
/// ```
/// This works really nicely, because it keeps the code small and concise. There
/// is however, a problem with this code - it is not dynamic. A soon as there is
/// an unknown or non-fixed amount of distinct weights, we need to use a recursive
/// algorithm, because we cannot change our code at runtime (and code cannot be
/// infinitely long).
///
/// The recursive definition of an algorithm which does exactly this, is quite a
/// lot longer, but works on the fundamentally same level.
///
///
/// # Arguments
///
/// * `sums`: An empty hashmap of sums which are each linked to a combination
/// * `arr`: An array of all the multiples for each weight.
/// * `n`: the depth of the algorithm, begins at 0.
/// * `counters`: an empty vector, which will be used to track which combination
/// is being computed and expanded upon at the moment.
///
/// returns: ()
///
/// # Examples
///
/// ```
/// let arr = read_weights(&file);
///
/// let mut sums: HashMap<i64, Vec<i64>> = HashMap::new();
/// let mut counters = vec![0; arr.len()];
/// recurse(&mut sums, &arr, 0, &mut counters);
/// //Fills the hashmap with the sums of every combinations
/// ```
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

/// Reads the weights and their respective amount from a file, and generates an 2D-array with
/// all the weights and their possible states.
///
/// # Arguments
///
/// * `file`: the file to read the weights from
///
/// returns: Vec<Vec<i64, Global>, Global>
///
/// # Examples
///
/// ### The file : input.txt
/// ```
/// 3
/// 2 1
/// 3 2
/// 6 2
/// ```
///
/// ### Your code
/// ```
/// let weights = read_weights("input.txt");
///
/// //weights: [[-2,0,2],
/// //          [-6,-3,0,3,6],
/// //          [-12,-6,0,6,12]]
///
/// ```
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

/// Generates an array of all multiples of the given weight from **-count\*weight** up to **count\*weight**
///
/// # Arguments
///
/// * `count`: The amount of weights with the given weight
/// * `weight`: The weight of the weight
///
/// returns: Vec<i64, Global>
///
/// # Examples
///
/// Given a weight of 2, and a count of 3, for example when reading the line '2 3' from an input file.
/// ```
/// let weight_perms = generate_weight_perms(3, 2);
/// //weight_perms: [-6,-4,-2,0,2,4,6]
/// ```
fn generate_weight_perms(count: i64, weight: i64) -> Vec<i64> {
    let mut weights = Vec::new();
    for i in (-count)..(count + 1) {
        weights.push(i * weight);
    }
    return weights;
}

/// A wrapper object that is used to represent a List of all sums and and
/// a valid combination that makes them up.
struct Solution {
    values: Vec<SolutionSlice>,
}

impl Solution {
    /// Creates a Solution wrapper from a hashmap of sums that specifies where
    /// each weight of has to go on the scale.
    ///
    /// # Arguments
    ///
    /// * `sums`: Hashmap with a combination that makes up a sum
    /// * `arr`: The array that contains each permutation of each weight
    /// according to how often it can be used. See #generate_weight_perms()
    ///
    /// returns: Solution
    ///
    /// # Examples
    ///
    /// ```
    ///  let mut sums: HashMap<i64, Vec<i64>> = HashMap::new();
    ///  let mut counters = vec![0; arr.len()];
    ///  recurse(&mut sums, &arr, 0, &mut counters); //Run algorithm generating sums
    ///
    ///  let solution = Solution::create(&sums, &arr); //Create the solution object
    /// ```
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

    /// Writes the solution to a file
    ///
    /// # Arguments
    ///
    /// * `file_name`: Path of the file to write to
    /// * `interpolate`: Whether to interpolate for values,
    /// where not combination exists that sums up to that value.
    /// If this is set to true, the combination whose sum is closest to the value is returned.
    ///
    /// returns: ()
    ///
    /// # Examples
    ///
    /// ```
    /// write_to_file("ergebnisse/loesung0.txt", true);
    /// //Creates the file 'ergebnisse/loesung0.txt' and writes the combinations
    /// //for each value to be measured on the scale.
    /// ```
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
                            sum, interpolated, left, right
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

    /// Returns the combination whose sum is closest to the given weight.
    ///
    /// # Arguments
    ///
    /// * `weight`: Weight to search for
    ///
    /// returns: (i64, Vec<i64, Global>, Vec<i64, Global>)
    ///
    /// # Examples
    ///
    /// ```
    /// let sol = Solution::create(); //Sums:[1,4,7]
    /// sol.get_closest_combination(5); //->4
    /// ```
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
