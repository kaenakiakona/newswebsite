# Use a Rust base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /app

# Copy the Rust project files to the container
COPY . /app/

#expose port 8080
EXPOSE 8080

# Build the Rust project
RUN cargo build --release

# Set the command to run the built project

CMD ["cargo", "run", "--release"]

#changes we tried: 
# get rid of RUN line and replace CMD line with CMD ["cargo", "build", "--release"]