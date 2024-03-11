
extern crate clap;
use clap::Parser;


use std::path::PathBuf;

mod histogram_specs;
use histogram_specs::HistogramSpecs;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    
    filename_1: PathBuf, // First file for loading data

    filename_2: PathBuf, // Second file, might be the same

    #[arg(short, long, value_name="DELTA_R", default_value_t=1.0)]
    delta_r: f64,

    #[arg(short, long, value_name="MAX_R", default_value_t=50.0)]
    max_r: f64,


}

fn main() {
    
    let args = Args::parse();

    println!("{}", args.filename_1 == args.filename_2);


    /* Calculate parameters for histogramming */

    // Factor from r^2 to bin index
    let bin_factor = 1.0/(args.delta_r*args.delta_r);

    let square_bins = ((args.max_r / args.delta_r).powf(2.0)) as usize;

    let linear_bins = (args.max_r / args.delta_r) as usize;

    //println!("Linear bins = {}, Square bins = {}", linear_bins, square_bins);

    let histogram_data = HistogramSpecs {
        bin_size: args.delta_r, 
        bin_factor: bin_factor,
        n_linear_bins: linear_bins,
        n_square_bins: square_bins
    };
    
}
