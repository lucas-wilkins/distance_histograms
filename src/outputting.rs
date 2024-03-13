/* Output formatting and writing */

use crate::histogram_specs::HistogramSpecs;

pub fn format_histogram_data(hist_data: &Vec<u64>, histogram_specs: &HistogramSpecs) -> String {
    let mut lines: Vec<String> = Vec::new();
    
    for i in 0..hist_data.len() {
        let data = hist_data[i];
        let f: f64 = i as f64;
        let start_r = f*histogram_specs.bin_size;
        let end_r = (f + 1.0)*histogram_specs.bin_size;

        lines.push(format!("{}, {}, {}", start_r, end_r, data));
    }

    lines.join("\n")
}

