# llm_backend
The LLM backend of the app

If your machine has Metal support (Macbook) then you can add the "metal" feature to the kalosm crate.
If your machine has CUDA support (NVIDIA GPU) then you can add the "cuda" feature to the kalosm crate.

# Usage
curl localhost:8000/chat/completions -d '{"type":"structured","data":"hello"}' -H "Content-Type: applications/json"

curl localhost:8000/chat/completions -d '{"type":"chat","prompt":"hello"}' -H "Content-Type: Applications/json"


Postman

# POST
http://localhost:8000/chat/completions

# Body for chat
{
    "type" : "chat",
    "prompt" : "DATA"
}
# body for structured
{
    "type" : "structured",
    "data" : "DATA"
}


[profile.release]
lto = true
codegen-units = 1


# Test run code

curl localhost:8000/chat/completions -d '{"type":"structured","data":"I have a web application which is connected to a backend."}' -H "Content-Type: applications/json"