#!/bin/bash

# Make sure the Python script is executable
chmod +x benchmark_comparison.py

python -m venv benchmark_venv
source benchmark_venv/bin/activate

# Install required dependencies (if not already installed)
pip install beautifulsoup4 pandas matplotlib seaborn numpy

# Run the script (assumes you're in the root of your rust-loguru project)
./benchmark_comparison.py

# Open the results
echo "Benchmark results generated in benchmark_plots directory"
echo "HTML summary: benchmark_plots/benchmark_summary.html"

# Deactivate the virtual environment
deactivate

# Remove the virtual environment
rm -rf benchmark_venv