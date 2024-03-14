
extern crate clap;
use clap::Parser;


use compound_duration::format_dhms;

use std::path::PathBuf;
use std::{thread, thread::available_parallelism};
use std::iter::zip;
use std::time::Instant;

mod histogram_specs;
use histogram_specs::HistogramSpecs;

mod file_loading;
use file_loading::{load_data, XYZData};

mod outputting;
use outputting::format_histogram_data;

/* Argument parser specification */

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    
    filename_1: PathBuf, // First file for loading data

    filename_2: PathBuf, // Second file, might be the same

    #[arg(short, long, value_name="DELTA_R", default_value_t=1.0)]
    delta_r: f64,

    #[arg(short, long, value_name="MAX_R", default_value_t=50.0)]
    max_r: f64,

    #[arg(short, long, value_name="LIN_THREADS", default_value_t=5)]
    n_threads: u8,

    #[arg(long, value_name="ASSUME_DIFFERENT")]
    assume_different: bool,

    #[arg(short, long, value_name="TIMER")]
    timer: bool,

    #[arg(short, long, value_name="OUTPUT_FILE")]
    output: Option<String>,

    #[arg(short, long, value_name="VERBOSE")]
    verbose: bool
}

/*
 *
 *   Main
 *
 */

fn main() {
    
    // command line arguments
    let args = Args::parse();


    // Start timer
    let now = Instant::now();

    let verbose =args.verbose;

    // Is it a diagonally symmetric problem, if so, we can optimise
    let diagonal = (args.filename_1 == args.filename_2) && !args.assume_different;

    if verbose {
        let parallelism = available_parallelism().unwrap().get();
        println!("{} cores available", parallelism); 
        if diagonal {
            println!("Using diagonal method");
        } else {
            println!("Using off-diagonal method");
        }
    }

    /* Calculate parameters for histogramming */

    // Factor from r^2 to bin index
    let bin_factor = 1.0/(args.delta_r*args.delta_r); 
    let bin_scale = 1.0/args.delta_r;   // Scale for coordinates

    let square_bins = ((args.max_r / args.delta_r).powf(2.0)) as usize; //number of bins in the r squared representation

    let linear_bins = (args.max_r / args.delta_r) as usize; // number of bins in the linear representation

    // Object that holds information about the specs
    let histogram_specs: HistogramSpecs = HistogramSpecs {
        bin_size: args.delta_r, 
        bin_factor: bin_factor,
        n_linear_bins: linear_bins,
        n_square_bins: square_bins
    };

    
    
    // This will be for collecting up the thread data
    let mut square_histogram = histogram_specs.create_empty_histogram();

    /* Two different methods, one if it is a diagonal entry, another if it is not */
    if diagonal {

        /*
         *
         *    Diagonal branch
         *
         */


        let data: XYZData = load_data(&args.filename_1, bin_scale);
    

        let n = args.n_threads as usize;

        let chunk_edges: Vec<usize> = (0..=n).map(|i| ((i as u64) * (data.n_points as u64) / (n as u64)) as usize).collect();

        /* Create list of ranges in a triangular grid */
        let mut grid_corners_indices: Vec<(usize, usize, usize, usize, bool)> = Vec::new(); 

        for i in 0..n {
            for j in 0..=i {
                grid_corners_indices.push(
                    (chunk_edges[i],  // dimension 1
                    chunk_edges[i+1], //
                    chunk_edges[j],   // dimension 2
                    chunk_edges[j+1], //
                    i==j              // is it a chunk on the diagonal
                ))
            }
        }

        let threads: Vec<_> = grid_corners_indices.iter().map(|(start_1, end_1, start_2, end_2, is_on_diagonal)| {

            let start_1 = start_1.clone();
            let end_1 = end_1.clone();
            let start_2 = start_2.clone();
            let end_2 = end_2.clone();
            let is_on_diagonal = is_on_diagonal.clone();
            
            let mut histogram_values = histogram_specs.create_empty_histogram();
            let raw = data.data.clone();
            
            thread::spawn(move || {

                if verbose {
                    if is_on_diagonal {
                        println!("Starting thread for diagonal chunk {start_1}..{end_1} x {start_2}..{end_2}");
                    } else {
                        println!("Starting thread for off-diagonal chunk {start_1}..{end_1} x {start_2}..{end_2}");
                    }
                }

                /* Thread code */

                if is_on_diagonal {
                    for index_1 in start_1..end_1 {
                        for index_2 in start_2..index_1 { // note we don't use end_2 here, don't include the actual diagonal
                            let i1 = 3*index_1;
                            let i2 = 3*index_2;
                            
                            let dx = raw[i1] - raw[i2];
                            let dy = raw[i1+1] - raw[i2+1];
                            let dz = raw[i1+2] - raw[i2+2];

                            let bin = (dx*dx + dy*dy + dz*dz) as usize;

                            histogram_values[bin] += 2; // Add two because its off diagonal
                            
                        }
                    }

                    // Diagonal elements will be in bin zero, so
                    histogram_values[0] += (end_1 - start_1) as u64;
                    
                    
                    
                } else {
                    for index_1 in start_1..end_1 {
                        for index_2 in start_2..end_2 {
                            let i1 = 3*index_1;
                            let i2 = 3*index_2;
                            
                            let dx = raw[i1] - raw[i2];
                            let dy = raw[i1+1] - raw[i2+1];
                            let dz = raw[i1+2] - raw[i2+2];

                            let bin = (dx*dx + dy*dy + dz*dz) as usize;

                            histogram_values[bin] += 2; // Add two because its off diagonal
                        }
                    }

                }


                if verbose {
                    println!("Finished chunk {start_1}..{end_1} x {start_2}..{end_2}");
                }

                histogram_values
                
            })
        }).collect(); // If we don't collect, they wont spawn
    
        /* Combine the output of all the arrays */

        for thread in threads {
            let contribution: Vec<u64> = thread.join().unwrap();

            for i in 0 .. contribution.len() {
                square_histogram[i] += contribution[i];
            }

        }
        

    } else {

        /*
         *
         * Off-diagonal branch 
         * 
         */

        let data_1: XYZData = load_data(&args.filename_1, bin_scale);
        let data_2: XYZData = load_data(&args.filename_2, bin_scale);
    

        /* Create list of ranges for each thread in a rectangular grid */
        let n_1: usize = args.n_threads as usize;
        let n_2: usize = args.n_threads as usize;
    
        let points_1: Vec<usize> = (0..=n_1).map(|i| ((i as u64) * (data_1.n_points as u64) / (n_1 as u64)) as usize).collect();
        let points_2: Vec<usize> = (0..=n_2).map(|i| ((i as u64) * (data_2.n_points as u64) / (n_2 as u64)) as usize).collect();

        let pairs_1 = zip(points_1.clone(), points_1[1..].to_vec());
        let pairs_2 = zip(points_2.clone(), points_2[1..].to_vec());

        let pair_pairs = 
            pairs_1.clone().map(|(a,b)| {
                pairs_2.clone().map(move |(c, d)| {
                    (a, b, c, d)
                })
            }).flatten();



        /* Set up threads */

        let threads: Vec<_> = 
            pair_pairs.map(|(start_1, end_1, start_2, end_2)| {

                let mut histogram_values = histogram_specs.create_empty_histogram();

                let raw_1 = data_1.data.clone();
                let raw_2 = data_2.data.clone();

                thread::spawn(move || {

                    if verbose {
                        println!("Starting thread for chunk {start_1}..{end_1} x {start_2}..{end_2}");
                    }

                    /* Thread code */

                    for index_1 in start_1..end_1 {
                        for index_2 in start_2..end_2 {
                            let i1 = 3*index_1;
                            let i2 = 3*index_2;
                            
                            let dx = raw_1[i1] - raw_2[i2];
                            let dy = raw_1[i1+1] - raw_2[i2+1];
                            let dz = raw_1[i1+2] - raw_2[i2+2];

                            let bin = (dx*dx + dy*dy + dz*dz) as usize;

                            histogram_values[bin] += 1;
                        }
                    }


                    if verbose {
                        println!("Finished chunk {start_1}..{end_1} x {start_2}..{end_2}");
                    }

                    histogram_values
                    
                })
            }).collect(); // If we don't collect, they wont spawn


        /* Combine the output of all the arrays */

        for thread in threads {
            let contribution: Vec<u64> = thread.join().unwrap();

            for i in 0 .. contribution.len() {
                square_histogram[i] += contribution[i];
            }

        }
    }

    /* Final stages, organise data, summarise, output */

    let output = histogram_specs.unsquare_historgam(square_histogram);

    
    // Timing details

    if args.timer {
        let elapsed_time = now.elapsed();
        let n_pairs = output.iter().fold(0u64, |acc, x| acc+x);
        println!("Binned {} pairs in {}", n_pairs, format_dhms(elapsed_time.as_secs()));
    }

    /* Write data to file / stdout */

    let output_string = format_histogram_data(&output, &histogram_specs);

    match args.output {
        Some(filename) => match std::fs::write(filename, output_string) {
            Err(error) => println!("Failed to write output, {}", error),
            Ok(_) => ()
        }
        None => {println!("{}", output_string); }
    };

}
