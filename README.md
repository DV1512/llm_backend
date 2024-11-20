# llm_backend
The LLM backend of the app

If your machine has Metal support (Macbook) then you can add the "metal" feature to the kalosm crate.
If your machine has CUDA support (NVIDIA GPU) then you can add the "cuda" feature to the kalosm crate.



## Prompt Usage

# /templated
# Example:
curl localhost:8000/chat/templated -d '{"prompt_template":"What are the associated names for APT28?","name":"APT28"}' -H "Content-type: applications/json"

# /completions
# Example:
curl localhost:8000/chat/completions -d '{"prompt":"What are the associated names for APT28?"}' -H "Content-type: applications/json"
