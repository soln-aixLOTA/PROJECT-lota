"""This module defines the main entry point for the Apify Actor.

Feel free to modify this file to suit your specific needs.

To build Apify Actors, utilize the Apify SDK toolkit, read more at the official documentation:
https://docs.apify.com/sdk/python
"""

import os
import json
from pathlib import Path

# Beautiful Soup - A library for pulling data out of HTML and XML files. Read more at:
# https://www.crummy.com/software/BeautifulSoup/bs4/doc
from bs4 import BeautifulSoup

# HTTPX - A library for making asynchronous HTTP requests in Python. Read more at:
# https://www.python-httpx.org/
from httpx import AsyncClient

# Apify SDK - A toolkit for building Apify Actors. Read more at:
# https://docs.apify.com/sdk/python
from apify import Actor


async def main() -> None:
    """Main entry point for the Apify Actor."""
    async with Actor:
        # Create storage directories if they don't exist
        dataset_dir = Path('storage/datasets/default')
        dataset_dir.mkdir(parents=True, exist_ok=True)

        # Retrieve the input object for the Actor
        actor_input = await Actor.get_input() or {'url': 'https://apify.com/'}
        url = actor_input.get('url')
        print(f'Processing URL: {url}')

        # Create an asynchronous HTTPX client for making HTTP requests
        async with AsyncClient() as client:
            # Fetch the HTML content of the page
            print(f'Sending request to {url}')
            response = await client.get(url, follow_redirects=True)
            print(f'Received response: {response.status_code}')

        # Parse the HTML content using Beautiful Soup
        soup = BeautifulSoup(response.content, 'lxml')
        print('Parsed HTML content successfully')

        # Extract all headings from the page
        headings = []
        for heading in soup.find_all(['h1', 'h2', 'h3', 'h4', 'h5', 'h6']):
            heading_object = {'level': heading.name, 'text': heading.text.strip()}
            print(f'Found heading: {heading_object}')
            headings.append(heading_object)

        print(f'Found {len(headings)} headings')

        # Save the extracted headings to both dataset and key-value store
        await Actor.push_data(headings)
        await Actor.set_value('OUTPUT', {'headings': headings})

        # Also save to a local file for debugging
        output_file = dataset_dir / 'results.json'
        with open(output_file, 'w') as f:
            json.dump(headings, f, indent=2)
        print(f'Saved results to {output_file}')
