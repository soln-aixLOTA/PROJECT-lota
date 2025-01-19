"""This module defines the main entry point for the Apify Actor.

Feel free to modify this file to suit your specific needs.

To build Apify Actors, utilize the Apify SDK toolkit, read more at the official documentation:
https://docs.apify.com/sdk/python
"""

import os
import sys
import json
import logging
from pathlib import Path

# Beautiful Soup - A library for pulling data out of HTML and XML files. Read more at:
# https://www.crummy.com/software/BeautifulSoup/bs4/doc
from bs4 import BeautifulSoup

# HTTPX - A library for making asynchronous HTTP requests in Python. Read more at:
# https://www.python-httpx.org/
from httpx import AsyncClient, HTTPError

# Apify SDK - A toolkit for building Apify Actors. Read more at:
# https://docs.apify.com/sdk/python
from apify import Actor

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(message)s',
    handlers=[logging.StreamHandler(sys.stdout)]
)
logger = logging.getLogger(__name__)

async def main() -> None:
    """Main entry point for the Apify Actor."""
    try:
        async with Actor:
            # Create storage directories if they don't exist
            dataset_dir = Path('storage/datasets/default')
            dataset_dir.mkdir(parents=True, exist_ok=True)
            logger.info(f'Created dataset directory: {dataset_dir}')

            # Retrieve the input object for the Actor
            actor_input = await Actor.get_input() or {'url': 'https://apify.com/'}
            url = actor_input.get('url')
            logger.info(f'Processing URL: {url}')

            try:
                # Create an asynchronous HTTPX client for making HTTP requests
                async with AsyncClient() as client:
                    # Fetch the HTML content of the page
                    logger.info(f'Sending request to {url}')
                    response = await client.get(url, follow_redirects=True)
                    response.raise_for_status()
                    logger.info(f'Received response: {response.status_code}')

                # Parse the HTML content using Beautiful Soup
                soup = BeautifulSoup(response.content, 'lxml')
                logger.info('Parsed HTML content successfully')

                # Extract all headings from the page
                headings = []
                for heading in soup.find_all(['h1', 'h2', 'h3', 'h4', 'h5', 'h6']):
                    heading_object = {'level': heading.name, 'text': heading.text.strip()}
                    logger.debug(f'Found heading: {heading_object}')
                    headings.append(heading_object)

                logger.info(f'Found {len(headings)} headings')

                # Save the extracted headings to both dataset and key-value store
                await Actor.push_data(headings)
                logger.info('Pushed data to dataset')

                await Actor.set_value('OUTPUT', {'headings': headings})
                logger.info('Set value in key-value store')

                # Also save to a local file for debugging
                output_file = dataset_dir / 'results.json'
                with open(output_file, 'w') as f:
                    json.dump(headings, f, indent=2)
                logger.info(f'Saved results to {output_file}')

            except HTTPError as e:
                logger.error(f'HTTP error occurred: {e}')
                raise
            except Exception as e:
                logger.error(f'An error occurred while processing the page: {e}')
                raise

    except Exception as e:
        logger.error(f'Actor failed: {e}')
        sys.exit(1)

if __name__ == '__main__':
    import asyncio
    asyncio.run(main())
