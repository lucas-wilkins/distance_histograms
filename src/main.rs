
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

fn main() {
    
    let args = Args::parse();


    // Start timer
    let now = Instant::now();

    let verbose =args.verbose;

    if verbose {
        let parallelism = available_parallelism().unwrap().get();
        println!("{} cores available", parallelism); 
    }

    /* Calculate parameters for histogramming */

    // Factor from r^2 to bin index
    let bin_factor = 1.0/(args.delta_r*args.delta_r);
    let bin_scale = 1.0/args.delta_r;

    let square_bins = ((args.max_r / args.delta_r).powf(2.0)) as usize;

    let linear_bins = (args.max_r / args.delta_r) as usize;

    //println!("Linear bins = {}, Square bins = {}", linear_bins, square_bins);

    let histogram_specs: HistogramSpecs = HistogramSpecs {
        bin_size: args.delta_r, 
        bin_factor: bin_factor,
        n_linear_bins: linear_bins,
        n_square_bins: square_bins
    };

    let data_1: XYZData = load_data(&args.filename_1, bin_scale);
    let data_2: XYZData = load_data(&args.filename_2, bin_scale);

    
    /* Create list of ranges for each thread */

    let n_1: usize = args.n_threads as usize;
    let n_2: usize = args.n_threads as usize;

    let points_1: Vec<usize> = (0..=n_1).map(|i| (i * data_1.n_points) / n_1).collect();
    let points_2: Vec<usize> = (0..=n_2).map(|i| (i * data_2.n_points) / n_2).collect();

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
    let mut square_histogram = histogram_specs.create_empty_histogram();

    for thread in threads {
        let contribution: Vec<u64> = thread.join().unwrap();

        for i in 0 .. contribution.len() {
            square_histogram[i] += contribution[i];
        }

    }

    let output = histogram_specs.unsquare_historgam(square_histogram);

    dbg!(output);

    // Timing details

    if args.timer {
        let elapsed_time = now.elapsed();
        let n_pairs = (data_1.n_points as u64) * (data_2.n_points as u64);
        println!("Binned {} pairs in {}", n_pairs, format_dhms(elapsed_time.as_secs()));
    }
}
