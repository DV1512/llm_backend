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
curl localhost:8000/chat/completions -d '{"type":"chat","prompt":"I have a web applciation which is connected to a beackend."}' -H "Content-Type: applications/json"


# new test string
curl localhost:8000/chat/completions -d '{"type":"structured","data":"I have the following components: A web interface, A llm, database. The web interface is connected to the LLM, the LLM has access to user data on the database. Do threat moddeling on this and give me 10 threats and mitigations."}' -H "Content-Type: applications/json"


# Chat testing prompted responses and threat modelling
# Start up LLM
# Run the below in a terminal
curl localhost:8000/chat/completions -d '{"type":"chat","prompt":"Hello I would like to do threatmodelling on a web page that is connected to an LLM, using a chatwindow to answer user questions."}' -H "Content-Type: applications/json"

curl localhost:8000/chat/completions -d '{"type":"chat","prompt":"I would like to do an overall analysis."}' -H "Content-Type: applications/json"