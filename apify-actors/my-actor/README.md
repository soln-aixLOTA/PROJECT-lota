## Scrape single-page in Python template

A template for [web scraping](https://apify.com/web-scraping) data from a single web page in Python. The URL of the web page is passed in via input, which is defined by the [input schema](https://docs.apify.com/platform/actors/development/input-schema). The template uses the [HTTPX](https://www.python-httpx.org) to get the HTML of the page and the [Beautiful Soup](https://www.crummy.com/software/BeautifulSoup/bs4/doc/) to parse the data from it. The data are then stored in a [dataset](https://docs.apify.com/sdk/python/docs/concepts/storages#working-with-datasets) where you can easily access them.

The scraped data in this template are page headings but you can easily edit the code to scrape whatever you want from the page.

## Included features

- **[Apify SDK](https://docs.apify.com/sdk/python/)** for Python - a toolkit for building Apify [Actors](https://apify.com/actors) and scrapers in Python
- **[Input schema](https://docs.apify.com/platform/actors/development/input-schema)** - define and easily validate a schema for your Actor's input
- **[Request queue](https://docs.apify.com/sdk/python/docs/concepts/storages#working-with-request-queues)** - queues into which you can put the URLs you want to scrape
- **[Dataset](https://docs.apify.com/sdk/python/docs/concepts/storages#working-with-datasets)** - store structured data where each object stored has the same attributes
- **[HTTPX](https://www.python-httpx.org)** - library for making asynchronous HTTP requests in Python
- **[Beautiful Soup](https://www.crummy.com/software/BeautifulSoup/bs4/doc/)** - library for pulling data out of HTML and XML files

## How it works

1. `Actor.get_input()` gets the input where the page URL is defined
2. `httpx.AsyncClient().get(url)` fetches the page
3. `BeautifulSoup(response.content, 'lxml')` loads the page data and enables parsing the headings
4. This parses the headings from the page and here you can edit the code to parse whatever you need from the page
    ```python
    for heading in soup.find_all(["h1", "h2", "h3", "h4", "h5", "h6"]):
    ```
5. `Actor.push_data(headings)` stores the headings in the dataset

## Resources

- [BeautifulSoup Scraper](https://apify.com/apify/beautifulsoup-scraper)
- [Python tutorials in Academy](https://docs.apify.com/academy/python)
- [Web scraping with Beautiful Soup and Requests](https://blog.apify.com/web-scraping-with-beautiful-soup/)
- [Beautiful Soup vs. Scrapy for web scraping](https://blog.apify.com/beautiful-soup-vs-scrapy-web-scraping/)
- [Integration with Make, GitHub, Zapier, Google Drive, and other apps](https://apify.com/integrations)
- [Video guide on getting scraped data using Apify API](https://www.youtube.com/watch?v=ViYYDHSBAKM)
- A short guide on how to build web scrapers using code templates:

[web scraper template](https://www.youtube.com/watch?v=u-i-Korzf8w)


## Getting started

For complete information [see this article](https://docs.apify.com/platform/actors/development#build-actor-locally). To run the actor use the following command:

```bash
apify run
```

## Deploy to Apify

### Connect Git repository to Apify

If you've created a Git repository for the project, you can easily connect to Apify:

1. Go to [Actor creation page](https://console.apify.com/actors/new)
2. Click on **Link Git Repository** button

### Push project on your local machine to Apify

You can also deploy the project on your local machine to Apify without the need for the Git repository.

1. Log in to Apify. You will need to provide your [Apify API Token](https://console.apify.com/account/integrations) to complete this action.

    ```bash
    apify login
    ```

2. Deploy your Actor. This command will deploy and build the Actor on the Apify Platform. You can find your newly created Actor under [Actors -> My Actors](https://console.apify.com/actors?tab=my).

    ```bash
    apify push
    ```

## Documentation reference

To learn more about Apify and Actors, take a look at the following resources:

- [Apify SDK for JavaScript documentation](https://docs.apify.com/sdk/js)
- [Apify SDK for Python documentation](https://docs.apify.com/sdk/python)
- [Apify Platform documentation](https://docs.apify.com/platform)
- [Join our developer community on Discord](https://discord.com/invite/jyEM2PRvMU)
