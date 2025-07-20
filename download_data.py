#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "requests",
#     "tqdm",
# ]
# ///

import re
import requests
import pathlib
import gzip
import shutil
import os
import argparse
from multiprocessing import Pool
from tqdm import tqdm

URL = "https://rebrickable.com/downloads/"
HEADERS = {'User-Agent': 'Mozilla/5.0'}

def process_file(args):
    """Worker function: downloads, decompresses, and cleans up a single file."""
    filename, file_url, download_dir = args
    gz_path = download_dir / filename
    csv_path = download_dir / filename.replace('.gz', '')

    try:
        # 1. Download the gzipped file
        response = requests.get(file_url, headers=HEADERS, timeout=60)
        response.raise_for_status()
        with open(gz_path, 'wb') as f:
            f.write(response.content)

        # 2. Decompress to CSV
        with gzip.open(gz_path, 'rb') as f_in:
            with open(csv_path, 'wb') as f_out:
                shutil.copyfileobj(f_in, f_out)

    except (requests.exceptions.RequestException, gzip.BadGzipFile) as e:
        # Use tqdm.write to print messages without breaking the progress bar
        tqdm.write(f"Failed processing {filename}: {e}")
    finally:
        # 3. Clean up the .gz file
        if gz_path.exists():
            os.remove(gz_path)

def main() -> int:
    """Finds all csv.gz links and processes them in parallel."""
    parser = argparse.ArgumentParser(
        description="Download and decompress all CSV data from Rebrickable.",
        formatter_class=argparse.RawTextHelpFormatter
    )
    parser.add_argument(
        '-d', '--directory',
        type=pathlib.Path,
        default='data',
        help="The directory to download files to.\n"
             "WARNING: This overwrites the contents with all of the data from Rebrickable."
    )
    args = parser.parse_args()
    download_dir = args.directory

    download_dir.mkdir(exist_ok=True)

    try:
        response = requests.get(URL, headers=HEADERS, timeout=10)
        response.raise_for_status()
    except requests.exceptions.RequestException as e:
        print(f"Error: Could not fetch page {URL}. {e}")
        return 1

    # Find all file URLs using the specified regular expression
    expr = re.compile(r'"(https?:\/\/cdn.rebrickable.com\/media\/downloads\/([a-zA-Z_\\]+\.csv\.gz)\?\d+\.\d+)"')
    links = {m.group(2): m.group(1) for m in expr.finditer(response.text)}

    if not links:
        print("Could not find any downloadable files.")
        return 1

    # Prepare tasks for the process pool, including the download directory
    tasks = [(filename, url, download_dir) for filename, url in links.items()]

    # Use a process pool to run tasks in parallel
    # `imap_unordered` processes items as they complete, feeding the progress bar
    with Pool() as pool:
        list(tqdm(pool.imap_unordered(process_file, tasks),
                  total=len(links),
                  desc="Downloading files",
                  unit="file"))
    return 0

if __name__ == "__main__":
    exit(main())
