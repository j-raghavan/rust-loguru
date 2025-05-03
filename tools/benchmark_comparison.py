#!/usr/bin/env python3
import os
import re
import glob
from bs4 import BeautifulSoup
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np
from pathlib import Path

# Set plot style
plt.style.use('ggplot')
sns.set_palette("Set2")

def parse_value_with_unit(value_str):
    """Parse a string like '562.42 ns' into value and unit."""
    if not value_str:
        return 0.0, ""

    match = re.match(r'([\d.]+)\s*(.+)', value_str.strip())
    if match:
        return float(match.group(1)), match.group(2).strip()
    return 0.0, ""

def normalize_to_ns(value, unit):
    """Convert time values to nanoseconds for consistent comparison."""
    if unit == "ns":
        return value
    elif unit == "�s" or unit == "us":
        return value * 1000.0
    elif unit == "ms":
        return value * 1000000.0
    elif unit == "s":
        return value * 1000000000.0
    return value

def normalize_throughput(value, unit):
    """Normalize throughput to elements per second."""
    if "Melem/s" in unit:
        return value * 1000000.0
    elif "Kelem/s" in unit:
        return value * 1000.0
    return value

def format_time(ns_value):
    """Format time values with appropriate units."""
    if ns_value < 1000:
        return f"{ns_value:.2f} ns"
    elif ns_value < 1000000:
        return f"{ns_value/1000:.2f} �s"
    elif ns_value < 1000000000:
        return f"{ns_value/1000000:.2f} ms"
    else:
        return f"{ns_value/1000000000:.2f} s"

def format_throughput(elem_per_sec):
    """Format throughput values with appropriate units."""
    if elem_per_sec < 1000:
        return f"{elem_per_sec:.2f} elem/s"
    elif elem_per_sec < 1000000:
        return f"{elem_per_sec/1000:.2f} Kelem/s"
    else:
        return f"{elem_per_sec/1000000:.2f} Melem/s"

def parse_criterion_report(html_file):
    """Parse a Criterion benchmark report HTML file."""
    try:
        with open(html_file, 'r', encoding='utf-8') as f:
            content = f.read()
    except UnicodeDecodeError:
        # Try with a different encoding if UTF-8 fails
        with open(html_file, 'r', encoding='latin-1') as f:
            content = f.read()

    soup = BeautifulSoup(content, 'html.parser')

    # Extract benchmark name from h2 tag
    benchmark_name = ""
    h2_tag = soup.find('h2')
    if h2_tag:
        benchmark_name = h2_tag.text.strip()

    # Skip if benchmark name is not found
    if not benchmark_name:
        return None

    # Parse benchmark name to extract logger and test type
    parts = benchmark_name.split('/')
    if len(parts) < 3:
        # Not a logger benchmark
        return None

    logger_type = parts[1].strip()
    test_type = '/'.join(parts[2:]).strip()  # Join remaining parts as test type

    # Extract metrics from the additional_stats table
    metrics = {}
    stats_tables = soup.select('.additional_stats table')
    if stats_tables:
        rows = stats_tables[0].select('tbody tr')
        for row in rows:
            cells = row.find_all('td')
            if len(cells) >= 3:
                metric_name = cells[0].text.strip()
                # Use the estimate column (middle column)
                metric_value = cells[1].text.strip()  # Lower bound
                estimate = cells[2].text.strip()      # Estimate
                upper_bound = cells[3].text.strip() if len(cells) > 3 else ""  # Upper bound

                metrics[metric_name] = estimate

    result = {
        'benchmark_name': benchmark_name,
        'logger': logger_type,
        'test_type': test_type,
    }

    # Extract and normalize key metrics
    if 'Mean' in metrics:
        mean_value, mean_unit = parse_value_with_unit(metrics['Mean'])
        result['mean_value'] = mean_value
        result['mean_unit'] = mean_unit
        result['mean_ns'] = normalize_to_ns(mean_value, mean_unit)

    if 'Median' in metrics:
        median_value, median_unit = parse_value_with_unit(metrics['Median'])
        result['median_value'] = median_value
        result['median_unit'] = median_unit
        result['median_ns'] = normalize_to_ns(median_value, median_unit)

    if 'Throughput' in metrics:
        throughput_value, throughput_unit = parse_value_with_unit(metrics['Throughput'])
        result['throughput_value'] = throughput_value
        result['throughput_unit'] = throughput_unit
        result['throughput_normalized'] = normalize_throughput(throughput_value, throughput_unit)

    return result

def find_criterion_reports(base_dir):
    """Recursively find all Criterion benchmark report index.html files."""
    report_files = []
    for root, dirs, files in os.walk(base_dir):
        if 'index.html' in files and 'report' in root:
            report_path = os.path.join(root, 'index.html')
            report_files.append(report_path)
    return report_files

def extract_numeric_value(test_type):
    """Extract numeric value from test type for sorting."""
    match = re.search(r'(\d+)$', test_type)
    if match:
        return int(match.group(1))
    return 0

def custom_sort_key(item):
    """Custom sort key that handles mixed string/numeric values."""
    match = re.search(r'(\d+)$', item)
    if match:
        # If the string ends with numbers, extract the prefix and the number
        prefix = item[:match.start()]
        number = int(match.group(1))
        # Return a tuple for sorting (prefix first, then number)
        return (prefix, number)
    # If no number at the end, just return the string with a 0
    return (item, 0)

def create_comparison_plots(df, output_dir):
    """Create comparison plots for the benchmark data."""
    os.makedirs(output_dir, exist_ok=True)

    # Get unique benchmark categories
    benchmark_categories = df['test_category'].unique()

    for category in benchmark_categories:
        category_df = df[df['test_category'] == category]

        # Group by test_subcategory for sorting and plotting
        test_subcategories = category_df['test_subcategory'].unique()

        # Sort subcategories using custom sort key
        sorted_subcategories = sorted(test_subcategories, key=custom_sort_key)

        logger_types = category_df['logger'].unique()

        # 1. Mean execution time comparison
        plt.figure(figsize=(12, 8))
        bar_width = 0.2
        index = np.arange(len(sorted_subcategories))

        for i, logger in enumerate(logger_types):
            logger_data = []
            for subcat in sorted_subcategories:
                data = category_df[(category_df['logger'] == logger) &
                                   (category_df['test_subcategory'] == subcat)]
                if not data.empty and 'mean_ns' in data.columns:
                    logger_data.append(data['mean_ns'].values[0])
                else:
                    logger_data.append(0)

            plt.bar(index + i*bar_width, logger_data, bar_width, label=logger)

        plt.xlabel('Test Parameters')
        plt.ylabel('Mean Execution Time (ns)')
        plt.title(f'{category} - Mean Execution Time Comparison')
        #plt.xticks(index + bar_width * (len(logger_types) - 1) / 2, sorted_subcategories)
        plt.xticks(
            index + bar_width * (len(logger_types) - 1) / 2,
            sorted_subcategories,
            rotation=45,
            ha='right',
            fontsize=10
        )
        plt.legend()
        plt.tight_layout()
        plt.savefig(os.path.join(output_dir, f'{category}_mean_time.png'), dpi=300)
        plt.close()

        # 2. Throughput comparison
        plt.figure(figsize=(12, 8))

        for i, logger in enumerate(logger_types):
            logger_data = []
            for subcat in sorted_subcategories:
                data = category_df[(category_df['logger'] == logger) &
                                   (category_df['test_subcategory'] == subcat)]
                if not data.empty and 'throughput_normalized' in data.columns:
                    logger_data.append(data['throughput_normalized'].values[0])
                else:
                    logger_data.append(0)

            plt.bar(index + i*bar_width, logger_data, bar_width, label=logger)

        plt.xlabel('Test Parameters')
        plt.ylabel('Throughput (elements/second)')
        plt.title(f'{category} - Throughput Comparison')
        #plt.xticks(index + bar_width * (len(logger_types) - 1) / 2, sorted_subcategories)
        plt.xticks(
            index + bar_width * (len(logger_types) - 1) / 2,
            sorted_subcategories,
            rotation=45,
            ha='right',
            fontsize=10
        )
        plt.legend()
        plt.tight_layout()
        plt.savefig(os.path.join(output_dir, f'{category}_throughput.png'), dpi=300)
        plt.close()

        # 3. Median execution time comparison
        plt.figure(figsize=(12, 8))

        for i, logger in enumerate(logger_types):
            logger_data = []
            for subcat in sorted_subcategories:
                data = category_df[(category_df['logger'] == logger) &
                                   (category_df['test_subcategory'] == subcat)]
                if not data.empty and 'median_ns' in data.columns:
                    logger_data.append(data['median_ns'].values[0])
                else:
                    logger_data.append(0)

            plt.bar(index + i*bar_width, logger_data, bar_width, label=logger)

        plt.xlabel('Test Parameters')
        plt.ylabel('Median Execution Time (ns)')
        plt.title(f'{category} - Median Execution Time Comparison')
        #plt.xticks(index + bar_width * (len(logger_types) - 1) / 2, sorted_subcategories)
        plt.xticks(
            index + bar_width * (len(logger_types) - 1) / 2,
            sorted_subcategories,
            rotation=45,
            ha='right',
            fontsize=10
        )
        plt.legend()
        plt.tight_layout()
        plt.savefig(os.path.join(output_dir, f'{category}_median_time.png'), dpi=300)
        plt.close()

def generate_summary_table(df, output_dir):
    """Generate a summary table of the benchmark results."""
    summary_data = []

    for category in df['test_category'].unique():
        category_df = df[df['test_category'] == category]
        for subcategory in sorted(category_df['test_subcategory'].unique(), key=custom_sort_key):
            subcat_df = category_df[category_df['test_subcategory'] == subcategory]

            row = {
                'Category': category,
                'Test': subcategory,
            }

            for logger in subcat_df['logger'].unique():
                logger_data = subcat_df[subcat_df['logger'] == logger]

                if not logger_data.empty:
                    if 'mean_ns' in logger_data.columns:
                        row[f'{logger} Mean'] = format_time(logger_data['mean_ns'].values[0])

                    if 'throughput_normalized' in logger_data.columns:
                        row[f'{logger} Throughput'] = format_throughput(logger_data['throughput_normalized'].values[0])

            summary_data.append(row)

    summary_df = pd.DataFrame(summary_data)

    # Save as CSV
    summary_df.to_csv(os.path.join(output_dir, 'benchmark_summary.csv'), index=False)

    # Save as HTML
    html_path = os.path.join(output_dir, 'benchmark_summary.html')
    with open(html_path, 'w') as f:
        f.write('<html><head><title>Benchmark Summary</title>')
        f.write('<style>table {border-collapse: collapse; width: 100%;} ')
        f.write('th, td {border: 1px solid #ddd; padding: 8px; text-align: left;} ')
        f.write('tr:nth-child(even) {background-color: #f2f2f2;} ')
        f.write('th {padding-top: 12px; padding-bottom: 12px; background-color: #4CAF50; color: white;}</style>')
        f.write('</head><body>')
        f.write('<h1>Benchmark Summary</h1>')
        f.write(summary_df.to_html(index=False))
        f.write('</body></html>')

    print(f"Summary table saved to {html_path}")

def main():
    # Configuration
    base_dir = '../target/criterion'  # Default location for Criterion results
    output_dir = 'benchmark_plots'  # Output directory for plots

    # Find all Criterion benchmark reports
    report_files = find_criterion_reports(base_dir)
    print(f"Found {len(report_files)} benchmark report files")

    # Parse all reports
    benchmark_data = []
    for report_file in report_files:
        data = parse_criterion_report(report_file)
        if data:
            benchmark_data.append(data)

    if not benchmark_data:
        print("No benchmark data found. Check the path to your Criterion results.")
        return

    # Convert to DataFrame
    df = pd.DataFrame(benchmark_data)

    # Extract test category and subcategory from test_type
    def extract_categories(test_type):
        parts = test_type.split('/')
        if len(parts) >= 2:
            return parts[0], '/'.join(parts[1:])
        return test_type, ""

    df['test_category'], df['test_subcategory'] = zip(*df['test_type'].apply(extract_categories))

    # Create comparison plots
    create_comparison_plots(df, output_dir)

    # Generate summary table
    generate_summary_table(df, output_dir)

    print(f"Benchmark comparison plots generated in '{output_dir}' directory")

if __name__ == "__main__":
    main()
