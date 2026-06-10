FROM rust:latest
RUN apt-get update && apt-get install -y git python3 && rm -rf /var/lib/apt/lists/*
RUN rustup component add rustfmt
WORKDIR /workspace
COPY . /rust-refactor-mcp
WORKDIR /rust-refactor-mcp
RUN cargo build --release
WORKDIR /test
COPY docker_test/run_tests.py /test/run_tests.py
RUN chmod +x /test/run_tests.py
ENTRYPOINT ["python3", "-u", "/test/run_tests.py"]
