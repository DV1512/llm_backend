#!/bin/bash

echo -n "Embedding and uploading MITRE ATT&CK techniques data... "
curl localhost:8000/embeddings -H "Content-Type: application/json" -d @data/techniques.json
echo "Done."

echo -n "Embedding and uploading MITRE ATT&CK mitigations data... "
curl localhost:8000/embeddings -H "Content-Type: application/json" -d @data/mitigations.json
echo "Done."
