# llm_backend
The LLM backend of the app

If your machine has Metal support (Macbook) then you can add the "metal" feature to the kalosm crate.
If your machine has CUDA support (NVIDIA GPU) then you can add the "cuda" feature to the kalosm crate.

# Explenation on how to run

*Termnial*

cargo build --release 
cargo run --release

*Curl request*

curl localhost:8000/chat/completions d '{"type":"", "keywords": [], "prompt": "hello"}' -H "Content-Type: applications/json"

*Type*

Type is either structured or chat

*keywords*

Can be left out or included.

***with: {type, keywords, prompt}, "keywords":["web","database","etc..."]***

***without: {type, prompt} no keywords***

# Example Terminal Run
`curl localhost:8000/chat/completions -d '{"type":"structured","keywords":[],"prompt":"Hello I would like to do threatmodelling on a web page that is connected to an LLM, using a chat window to answer user question Give me 5 threats and mitigations."}' -H "Content-Type: applications/json"`