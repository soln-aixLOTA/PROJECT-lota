#!/bin/bash

# Load environment variables from .env.local if it exists
if [ -f .env.local ]; then
    export $(cat .env.local | xargs)
fi

# Start the development server
npm run dev 