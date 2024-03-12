

pub struct HistogramSpecs {
    pub bin_size: f64, // Bin size 
    pub bin_factor: f64, // Amount to scale r^2 by to get the bin number
    pub n_linear_bins: usize, // Number of linear bins
    pub n_square_bins: usize, // Number of square bins
}

impl HistogramSpecs {
    fn create_empty_histogram(&self) -> Vec<u64> {
        /* Create an empty histogram for r^2 values */
        
        vec![0u64; self.n_square_bins]
    }

    fn unsquare_historgam(&self, data: Vec<u64>) -> Vec<u64> {
        /* Convert histogram of r^2 to linear */

        unimplemented!();
    }
}