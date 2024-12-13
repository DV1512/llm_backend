#!/bin/bash

curl localhost:8000/embeddings -H "Content-Type: application/json" -d @data/techniques.json
curl localhost:8000/embeddings -H "Content-Type: application/json" -d @data/mitigations.json
