# llm_backend
The LLM backend of the app

If your machine has Metal support (Macbook) then you can add the "metal" feature to the kalosm crate.
If your machine has CUDA support (NVIDIA GPU) then you can add the "cuda" feature to the kalosm crate.


# Example Terminal Run
´curl localhost:8000/chat/completions -d '{"type":"structured","keywords":[],"prompt":"Hello I would like to do threatmodelling on a web page that is connected to an LLM, using a chat window to answer user question Give me 5 threats and mitigations."}' -H "Content-Type: applications/json"´